import to_str::to_str;
import std::map::hashmap; 
import core::dvec::*;
import types::*;

// types
export subject, predicate, object, triple;

// this file
export to_str, create_store, each_triple;

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			reference(v)			{v}
			qref(v)				{#fmt["%?:%s", v.nindex, v.name]}
			typed_literal(v, t)	{#fmt["\"%s\"^^%s", v, t]}
			plain_literal(v, t)		{#fmt["\"%s\"@%s", v, t]}
			xml(v)					{v}
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
				result += #fmt["%?: %s  %s  ", i, sname, pname];
				
				alt entry.object
				{
					qref(q)
					{
						result += #fmt["%s}\n", get_friendly_name(self, q)];
					}
					_
					{
						result += #fmt["%s}\n", entry.object.to_str()];
					}
				}
			}
		};
		
		ret result;
	}
}

#[doc = "Initializes a store object."]
fn create_store(namespaces: [namespace]) -> store
{
	{
		namespaces: [{prefix: "", path: ""}] + [{prefix: "_", path: "_"}] + namespaces,
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
may refererence blank nodes that are already in the store). It's an error"]
fn add_triples(store: store, triples: [triple])
{
	for vec::each(triples)
	{|triple|
		let subject = make_qname(store, triple.subject);
		
		let entry = {
			predicate: make_qname(store, triple.predicate),
			object:
				alt triple.object
				{
					reference(r)
					{
						// References are always saved internally as qrefs (which should be
						// substantially more efficient than dealing with long URL strings).
						qref(make_qname(store, r))
					}
					_
					{
						triple.object
					}
				}};
		
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
			let obj =
				alt entry.object
				{
					qref(q)
					{
						reference(get_friendly_name(store, q))
					}
					_
					{
						entry.object
					}
				};
				
			let triple = {subject: copy(sname), predicate: get_friendly_name(store, entry.predicate), object: obj};
			if !callback(triple)
			{
				ret;
			}
		}
	};
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
