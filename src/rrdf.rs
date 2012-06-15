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
	ityped(str, qname),		// qname may be xsd:string
	istring(str, qname, str)	// string + type (eg string or normalizedString) + lang (which won't be empy)
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
			istring(v, q, l)	{#fmt["\"%s\"^^%?:%s@%s", v, q.nindex, q.name, l]}
		}
	}
}

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			{value: v, kind: "xsd:anyURI", lang: ""}		{v}
			{value: v, kind: "xsd:boolean", lang: ""}		{v}
			{value: v, kind: "xsd:string", lang: ""}		{#fmt["\"%s\"", v]}
			{value: v, kind: "xsd:string", lang: lang}	{#fmt["\"%s\"@%s", v, lang]}
			{value: v, kind: kind, lang: ""}				{#fmt["\"%s\"^^%s", v, kind]}
			{value: v, kind: kind, lang: lang}				{#fmt["\"%s\"^^%s@%s", v, kind, lang]}	// not sure this case makes sense
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

Note that the xsd, rdf, and rdfs namespaces are automatically added."]
fn create_store(namespaces: [namespace]) -> store
{
	{
		namespaces: [
			{prefix: "", path: ""},		// used for names that are not actually prefixed
			{prefix: "_", path: "_"},	// used for blank nodes
			{prefix: "xsd", path: "http://www.w3.org/2001/XMLSchema#"},
			{prefix: "rdf", path: "http://www.w3.org/1999/02/22-rdf-syntax-ns#"},
			{prefix: "rdfs", path: "http://www.w3.org/2000/01/rdf-schema#"}
		] + namespaces,
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
	alt vec::position(store.namespaces, {|e| str::len(e.path) > 1u && name.starts_with(e.path)})
	{
		option::some(i)
		{
			// name is an URL and it matches a path from namespaces
			{nindex: i, name: str::slice(name, str::len(store.namespaces[i].path), str::len(name))}
		}
		option::none
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
						fail(#fmt["%s doesn't match a path in namespaces and isn't a prefixed name", name])
					}
				};
				
			alt vec::position(store.namespaces, {|e| str::is_not_empty(e.path) && e.prefix == ns})
			{
				option::some(index)
				{
					// name seems to be a prefixed name and it matches a prefix from namespaces
					{nindex: index, name: suffix}
				}
				option::none
				{
					fail(#fmt["%s doesn't match a namespace from the store's namespaces", name])
				}
			}
		}
	}
}

fn make_iobject(store: store, object: object) -> iobject
{
	alt object
	{
		{value: v, kind: "xsd:anyURI", lang: ""}		{ireference(make_qname(store, v))}
		{value: v, kind: kind, lang: ""}				{ityped(v, make_qname(store, kind))}
		{value: v, kind: kind, lang: lang}				{istring(v, make_qname(store, kind), lang)}
	}
}

fn make_object(store: store, io: iobject) -> object
{
	alt io
	{
		ireference(value)
		{
			{value: get_friendly_name(store, value), kind: "xsd:anyURI", lang: ""}
		}
		ityped(value, kind)
		{
			{value: value, kind: get_friendly_name(store, kind), lang: ""}
		}
		istring(value, kind, lang)
		{
			{value: value, kind: get_friendly_name(store, kind), lang: lang}
		}
	}
}
