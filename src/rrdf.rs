import to_str::to_str;
import std::map::hashmap; 
import core::dvec::*;
import types::*;

// types
export subject, predicate, object, triple, solution;

// this file
export to_str, create_store, get_blank_name, add_triples, each_triple, compile;

type qname = {nindex: uint, name: str};				// nindex is into namespaces

// The value that is actually saved in the store.
enum iobject
{
	ireference(qname),
	ityped(str, qname),
	iplain(str, str)
}

type entry = {predicate: qname, object: iobject};

impl of to_str for iobject
{
	fn to_str() -> str
	{
		alt self
		{
			ireference(q)		{#fmt["%?:%s", q.nindex, q.name]}
			ityped(v, q)		{#fmt["\"%s\"^^%?:%s", v, q.nindex, q.name]}
			iplain(v, l)		{#fmt["\"%s\"@%s", v, l]}
		}
	}
}

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			reference(v)			{v}
			typed_literal(v, t)	{#fmt["\"%s\"^^%s", v, t]}
			plain_literal(v, t)		{#fmt["\"%s\"@%s", v, t]}
		}
	}
}

impl of to_str for triple
{
	fn to_str() -> str
	{
		#fmt["{%s, %s, %s}", self.subject, self.predicate, self.object.to_str()]
	}
}

impl of to_str for store
{
	fn to_str() -> str unsafe
	{
		let mut result = "";
		
		for vec::eachi(self.namespaces)
		{|i, ns|
			result += #fmt["%?: %s = %s\n", i, ns.prefix, ns.path];
		};
		
		result += "\n";
		
		for self.subjects.each()
		{|subject, entries|
			let sname = get_friendly_name(self, subject);
			for (*entries).eachi()
			{|i, entry|
				let pname = get_friendly_name(self, entry.predicate);
				result += #fmt["%?: %s  %s  %s}\n", i, sname, pname, entry.object.to_str()];
			}
		};
		
		ret result;
	}
}

#[doc = "Initializes a store object.

Note that the xsd namespace is automatically added."]
fn create_store(namespaces: [namespace]) -> store
{
	{
		namespaces: [
			{prefix: "", path: ""},
			{prefix: "_", path: "_"},
			{prefix: "xsd", path: "http://www.w3.org/2001/XMLSchema#"}] + namespaces,
		subjects: hashmap(hash_qn, eq_qn),
		mut next_blank: 0u
	}
}

#[doc = "Returns a unique name for a blank node in particular store.

Note that the prefix can be anything, including empty."]
fn get_blank_name(store: store, prefix: str) -> str
{
	let name = #fmt["_:%s-%?", prefix, copy(store.next_blank)];
	store.next_blank += 1u;
	ret name;
}

#[doc = "Adds new triples to the store.
 
Note that these triples are considered to already belong to the store (so they
may refererence blank nodes that are already in the store). It's an error to
use a prefixed URL name if the namespace was not registered with the store."]
fn add_triples(store: store, triples: [triple])
{
	for vec::each(triples)
	{|triple|
		let subject = make_qname(store, triple.subject);
		
		let entry = {
			predicate: make_qname(store, triple.predicate),
			object: make_iobject(store, triple.object)};
		
		alt store.subjects.find(subject)
		{
			option::some(entries)
			{
				(*entries).push(entry);
			}
			option::none
			{
				store.subjects.insert(subject, @dvec::from_vec([mut entry]));
			}
		}
	};
}

#[doc = "Calls the callback for each triple in the store (in an undefined order)."]
fn each_triple(store: store, callback: fn (triple) -> bool) unsafe
{
	for store.subjects.each()
	{|subject, entries|
		let sname = get_friendly_name(store, subject);
		for (*entries).each()
		{|entry|
			let obj = make_object(store, entry.object);
			let triple = {subject: copy(sname), predicate: get_friendly_name(store, entry.predicate), object: obj};
			if !callback(triple)
			{
				ret;
			}
		}
	};
}

#[doc = "Returns either a function capable of matching triples or a parse error.

Expr can be a subset of http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"SPARQL\"."]
fn compile(expr: str) -> result::result<selector, str>
{
	let parser = sparql::make_parser();
	result::chain_err(rparse::parse(parser, "sparql", expr))
	{|err|
		result::err(#fmt["%s on line %? col %?", err.mesg, err.line, err.col])
	}
}

// ---- Internal Functions ----------------------------------------------------
fn hash_qn(&&x: qname) -> uint                 {x.nindex + str::hash(x.name)}
fn eq_qn(&&a: qname, &&b: qname) -> bool {a.nindex == b.nindex && a.name == b.name}

fn get_friendly_name(store: store, qn: qname) -> str
{
	alt qn.nindex 
	{
		0u
		{
			copy(qn.name)
		}
		_
		{
			store.namespaces[qn.nindex].prefix + ":" + qn.name
		}
	}
}

fn get_full_name(store: store, qn: qname) -> str
{
	alt qn.nindex 
	{
		0u
		{
			copy(qn.name)
		}
		_
		{
			store.namespaces[qn.nindex].path + qn.name
		}
	}
}

fn make_qname(store: store, name: str) -> qname
{
	let (ns, suffix) =
		alt str::find_char(name, ':')
		{
			option::some(index)
			{
				(str::slice(name, 0u, index), str::slice(name, index + 1u, str::len(name)))
			}
			option::none
			{
				("", name)
			}
		};
		
	alt vec::position(store.namespaces, {|e| e.prefix == ns})
	{
		option::some(index)
		{
			{nindex: index, name: suffix}
		}
		option::none
		{
			fail(#fmt["%s is not a valid namespace", ns])
		}
	}
}

fn make_iobject(store: store, object: object) -> iobject
{
	alt object
	{
		reference(value)
		{
			ireference(make_qname(store, value))
		}
		typed_literal(value, kind)
		{
			ityped(value, make_qname(store, kind))
		}
		plain_literal(value, lang)
		{
			iplain(value, lang)
		}
	}
}

fn make_object(store: store, io: iobject) -> object
{
	alt io
	{
		ireference(value)
		{
			reference(get_friendly_name(store, value))
		}
		ityped(value, kind)
		{
			typed_literal(value, get_friendly_name(store, kind))
		}
		iplain(value, lang)
		{
			plain_literal(value, lang)
		}
	}
}
