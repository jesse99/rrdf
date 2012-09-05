//! The type which stores triples.
use core::dvec::*;
use std::map::*;
use object::*;
use solution::*;

export subject, predicate, triple, namespace, entry, extension_fn, store, create_store, make_triple_blank, 
	make_triple_str, make_triple_uri, store_trait, store_methods, to_str, base_iter, get_blank_name, contract_uri;
export expand_uri;			// this should be internal
export expr, pattern, variable, constant, triple_pattern, algebra, basic, group, optional, bind, filter,
	constant_expr, variable_expr, call_expr, extension_expr, query_context, object_to_str, get_object;

// --------------------------------------------------------------------------------------
// TODO: should be in expression.rs (see rust bug 3352)
enum expr
{
	constant_expr(object),
	variable_expr(~str),
	call_expr(~str, ~[@expr]),			// function name + arguments
	extension_expr(~str, ~[@expr])	// function name + arguments
}

// --------------------------------------------------------------------------------------
// TODO: should be in query.rs (see rust bug 3352)
enum pattern
{
	variable(~str),
	constant(object)
}

type triple_pattern = {subject: pattern, predicate: pattern, object: pattern};

enum algebra
{
	basic(triple_pattern),
	group(~[@algebra]),
	optional(@algebra),
	bind(expr, ~str),
	filter(expr)
}

type query_context =
	{
		namespaces: ~[namespace],
		extensions: @hashmap<~str, extension_fn>,
		algebra: algebra,
		order_by: ~[expr],
		distinct: bool,
		limit: Option<uint>,
		rng: rand::Rng,		// for RAND
		timestamp: Tm		// for NOW
	};

// --------------------------------------------------------------------------------------
// TODO: should be in object.rs (see rust bug 3352)
fn object_to_str(store: store, obj: object) -> ~str
{
	match obj
	{
		typed_value(value, kind) =>
		{
			fmt!("\"%s^^\"%s", value, contract_uri(store.namespaces, kind))
		}
		iri_value(iri) =>
		{
			let result = contract_uri(store.namespaces, iri);
			if result != iri
			{
				result
			}
			else
			{
				~"<" + iri + ~">"
			}
		}
		_ =>
		{
			obj.to_str()
		}
	}
}

// --------------------------------------------------------------------------------------
// TODO: should be in object.rs (see rust bug 3352)
fn get_object(row: solution_row, name: ~str) -> object
{
	match row.search(name)
	{
		option::Some(value) =>
		{
			value
		}
		option::None =>
		{
			unbound_value(name)
		}
	}
}

// --------------------------------------------------------------------------------------
/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
/// or a blank node (_1).
type subject = ~str;

/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date).
type predicate = ~str;

/// A relationship between a subject and an object.
/// 
/// * subject identifies a resource, usually via an IRI.
/// * predicate is an IRI describing the relationship. Also known as a property.
/// * object is a IRI, literal, or blank node containing the value for the relationship.
/// 
/// Here is a psuedo-code example:
/// 
/// ('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')
type triple = {subject: subject, predicate: predicate, object: object};

/// Name of a namespace plus the IRI it expands to.
type namespace = {prefix: ~str, path: ~str};

/// Predicate and object associated with a subject.
type entry = {predicate: ~str, object: object};

/// SPARQL extension function.
type extension_fn = fn@ (~[namespace], ~[object]) -> object;

/// Stores triples in a more or less efficient format.
type store = {
	namespaces: ~[namespace],
	subjects: hashmap<~str, @DVec<entry>>,
	extensions: @hashmap<~str, extension_fn>,
	next_blank: @mut int,
};

/// Initializes a store object.
/// 
/// xsd, rdf, rdfs, and owl namespaces are automatically added. An rrdf:pname extension is
/// automatically added which converts an iri_value to a string_value using namespaces (or
/// simply stringifies it if none of the namespaces paths match).
fn create_store(namespaces: ~[namespace], extensions: @hashmap<~str, extension_fn>) -> store
{
	extensions.insert(~"rrdf:pname", pname_fn);
	
	{
		namespaces: default_namespaces() + namespaces,
		subjects: std::map::str_hash(),
		extensions: extensions,
		next_blank: @mut 0,
	}
}

fn get_blank_name(store: store, prefix: ~str) -> ~str
{
	let suffix = *store.next_blank;
	*store.next_blank += 1;
	
	fmt!("_:%s-%?", prefix, suffix)
}

/// Returns either the iri or the prefixed version of the iri.
fn contract_uri(namespaces: ~[namespace], iri: ~str) -> ~str
{
	match vec::find(namespaces, |n| {str::starts_with(iri, n.path)})
	{
		option::Some(ns) =>
		{
			fmt!("%s:%s", ns.prefix, str::slice(iri, str::len(ns.path), str::len(iri)))
		}
		option::None =>
		{
			iri
		}
	}
}

trait store_trait
{
	fn add(subject: ~str, entries: ~[(~str, object)]);
	fn add_triple(namespaces: ~[namespace], triple: triple);
	fn add_aggregate(subject: ~str, predicate: ~str, label: ~str, entries: ~[(~str, object)]) -> ~str;
	fn add_alt(subject: ~str, values: ~[object]);
	fn add_bag(subject: ~str, values: ~[object]);
	fn add_container(subject: ~str, kind: ~str, values: ~[object]);
	fn add_list(subject: ~str, predicate: ~str, values: ~[object]);
	fn add_reify(subject: ~str, predicate: ~str, value: object);
	fn add_seq(subject: ~str, values: ~[object]);
	fn clear();
	fn find_object(subject: ~str, predicate: ~str) -> option::Option<object>;
	fn find_objects(subject: ~str, predicate: ~str) -> ~[object];
	fn replace_triple(namespaces: ~[namespace], triple: triple);
}

impl  store: store_trait 
{
	/// Efficient addition of triples to the store.
	/// 
	/// Typically create_int, create_str, etc functions are used to create objects.
	fn add(subject: ~str, entries: ~[(~str, object)])
	{
		if vec::is_not_empty(entries)
		{
			let subject = expand_uri_or_blank(self.namespaces, subject);
			let entries = vec::map(entries, |e| {expand_entry(self.namespaces, e)});
			match self.subjects.find(subject)
			{
				option::Some(list) =>
				{
					(*list).push_all(entries);
				}
				option::None =>
				{
					let list = @DVec();
					self.subjects.insert(subject, list);
					(*list).push_all(entries);
				}
			}
		}
	}
	
	/// Relatively inefficient addition of triples to the store.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn add_triple(namespaces: ~[namespace], triple: triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(subject)
		{
			option::Some(entries) =>
			{
				(*entries).push(entry);
			}
			option::None =>
			{
				self.subjects.insert(subject, @dvec::from_vec(~[mut entry]));
			}
		}
	}
	
	/// Adds a subject statement referencing a new blank node.
	/// 
	/// Label is an arbitrary string useful for debugging. Returns the name of the blank node.
	fn add_aggregate(subject: ~str, predicate: ~str, label: ~str, entries: ~[(~str, object)]) -> ~str
	{
		let blank = get_blank_name(self, label);
		self.add_triple(~[], {subject: subject, predicate: predicate, object: blank_value(blank)});
		self.add(blank, entries);
		return blank;
	}
	
	/// Adds statements representing a choice between alternatives.
	fn add_alt(subject: ~str, values: ~[object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt", values);
	}
	
	/// Adds statements representing an unordered set of (possibly duplicate) values.
	fn add_bag(subject: ~str, values: ~[object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag", values);
	}
	
	/// Adds statements representing an arbitrary open container using 1-based integral keys.
	fn add_container(subject: ~str, kind: ~str, values: ~[object])
	{
		let blank = get_blank_name(self, after(subject, ':') + "-items");
		self.add_triple(~[], {subject: subject, predicate: kind, object: blank_value(blank)});
		
		let predicates = do iter::map_to_vec(vec::len(values)) |i: uint| {fmt!("http://www.w3.org/1999/02/22-rdf-syntax-ns#_%?", i+1u)};
		self.add(blank, vec::zip(predicates, values));
	}
	
	/// Adds a fixed size list of (possibly duplicate) items.
	fn add_list(subject: ~str, predicate: ~str, values: ~[object])
	{
		let prefix = after(predicate, ':');
		let mut blank = get_blank_name(self, prefix);
		self.add_triple(~[], {subject: subject, predicate: predicate, object: blank_value(blank)});
		for vec::each(values)
		|value|
		{
			let next = get_blank_name(self, prefix);
			self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#first", object: value});
			self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: blank_value(next)});
			blank = next;
		};
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: iri_value(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#nil")});
	}
	
	/// Adds a statement about a statement.
	/// 
	/// Often used for meta-data, e.g. a timestamp stating when a statement was added to the store.
	fn add_reify(subject: ~str, predicate: ~str, value: object)
	{
		let mut blank = get_blank_name(self, after(predicate, ':'));
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#type", object: iri_value(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement")});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#subject", object: iri_value(subject)});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate", object: iri_value(predicate)});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#object", object: value});
	}
	
	/// Adds statements representing an ordered set of (possibly duplicate) values.
	fn add_seq(subject: ~str, values: ~[object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Seq", values);
	}
	
	/// Removes all triples from the store.
	fn clear()
	{
		// TODO: Replace this awful code once // https://github.com/mozilla/rust/issues/2775 is fixed.
		// (Tried making subjects mutable but that lead to illegal borrows all over the place).
		let mut keys = ~[];
		for self.subjects.each_key
		|key|
		{
			vec::push(keys, key);
		};
		
		for vec::each(keys)
		|key|
		{
			self.subjects.remove(key);
		};
	}
	
	/// Returns the first matching object, or option::none.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_object(subject: ~str, predicate: ~str) -> option::Option<object>
	{
		let subject = expand_uri_or_blank(self.namespaces, subject);
		let predicate = expand_uri(self.namespaces, predicate);
		
		match self.subjects.find(subject)
		{
			option::Some(entries) =>
			{
				match entries.position(|candidate| {candidate.predicate == predicate})
				{
					option::Some(index) =>
					{
						option::Some((*entries)[index].object)
					}
					option::None =>
					{
						option::None
					}
				}
			}
			option::None =>
			{
				option::None
			}
		}
	}
	
	/// Returns all matching objects.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_objects(subject: ~str, predicate: ~str) -> ~[object]
	{
		let subject = expand_uri_or_blank(self.namespaces, subject);
		let predicate = expand_uri(self.namespaces, predicate);
		
		match self.subjects.find(subject)
		{
			option::Some(entries) =>
			{
				do entries.get().filter_map		// TODO: pretty bad to call get, but dvec doesn't have filter_map atm
				|entry|
				{
					if entry.predicate == predicate
					{
						option::Some(entry.object)
					}
					else
					{
						option::None
					}
				}
			}
			option::None =>
			{
				~[]
			}
		}
	}
	
	/// Replaces the object of an existing triple or adds a new triple.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn replace_triple(namespaces: ~[namespace], triple: triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(subject)
		{
			option::Some(entries) =>
			{
				match entries.position(|candidate| {candidate.predicate == predicate})
				{
					option::Some(index) 	=> entries.set_elt(index, entry),
					option::None 			=> entries.push(entry),
				}
			}
			option::None =>
			{
				self.subjects.insert(subject, @dvec::from_vec(~[mut entry]));
			}
		}
	}
}

impl store : BaseIter<triple>
{
	/// Calls the blk for each triple in the store (in an undefined order).
	pure fn each(blk: fn(triple) -> bool)
	{
		unchecked		// TODO: remove once bug 3372 is fixed
		{
			for self.subjects.each()
			|subject, entries|
			{
				for (*entries).each()
				|entry|
				{
					let triple = {subject: subject, predicate: entry.predicate, object: entry.object};
					if !blk(triple)
					{
						return;
					}
				}
			};
		}
	}
	
	pure fn size_hint() -> Option<uint>
	{
		unchecked
		{
			option::Some(self.subjects.size())
		}
	}
}

impl  triple : ToStr 
{
	fn to_str() -> ~str
	{
		fmt!("{%s, %s, %s}", self.subject, self.predicate, self.object.to_str())
	}
}

impl  store : ToStr 
{
	fn to_str() -> ~str
	{
		let mut result = ~"";
		
		for self.subjects.each()
		|subject, entries|
		{
			for (*entries).eachi()
			|i, entry|
			{
				result += fmt!("%?: <%s>  <%s>  %s}\n", i, subject, entry.predicate, entry.object.to_str());
			}
		};
		
		return result;
	}
}

// ---- Private Functions -----------------------------------------------------
fn default_namespaces() -> ~[namespace]
{
	~[
		{prefix: ~"_", path: ~"_:"},
		{prefix: ~"xsd", path: ~"http://www.w3.org/2001/XMLSchema#"},
		{prefix: ~"rdf", path: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#"},
		{prefix: ~"rdfs", path: ~"http://www.w3.org/2000/01/rdf-schema#"},
		{prefix: ~"owl", path: ~"http://www.w3.org/2002/07/owl#"}
	]
}

fn expand_uri(namespaces: ~[namespace], name: ~str) -> ~str
{
	// TODO: need to % escape bogus characters (after converting to utf-8)
	for vec::each(namespaces)
	|namespace|
	{
		if str::starts_with(name, namespace.prefix + ":")
		{
			return namespace.path + str::slice(name, str::len(namespace.prefix) + 1u, str::len(name));
		}
	};
	return name;
}

fn expand_uri_or_blank(namespaces: ~[namespace], name: ~str) -> ~str
{
	if str::starts_with(name, "_:")
	{
		name
	}
	else
	{
		expand_uri(namespaces, name)
	}
}

fn expand_object(namespaces: ~[namespace], obj: object) -> object
{
	match obj
	{
		typed_value(value, kind) =>
		{
			typed_value(value, expand_uri(namespaces, kind))
		}
		iri_value(value) =>
		{
			iri_value(expand_uri(namespaces, value))
		}
		blank_value(value) =>
		{
			blank_value(expand_uri(namespaces, value))
		}
		_ =>
		{
			obj
		}
	}
}

fn expand_entry(namespaces: ~[namespace], entry: (~str, object)) -> entry
{
	{predicate: expand_uri(namespaces, entry.first()), object: expand_object(namespaces, entry.second())}
}

fn make_triple_blank(store: store, subject: ~str, predicate: ~str, value: ~str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: blank_value(fmt!("_:%s", value))
	}
}

fn make_triple_str(store: store, subject: ~str, predicate: ~str, value: ~str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: string_value(value, ~"")
	}
}

fn make_triple_uri(store: store, subject: ~str, predicate: ~str, value: ~str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: iri_value(expand_uri(store.namespaces, value))
	}
}

impl uint : BaseIter<uint>
{
	pure fn each(blk: fn(&&uint) -> bool)
	{
		let mut i = 0u;
		while i < self
		{
			if (!blk(i))
			{
				return;
			}
			i += 1u;
		}
	}
	
	pure fn size_hint() -> Option<uint>
	{
		option::Some(self)
	}
}

fn after(text: ~str, ch: char) -> ~str
{
	match str::rfind_char(text, ch)
	{
		option::Some(i) =>
		{
			str::slice(text, i+1u, str::len(text))
		}
		option::None =>
		{
			text
		}
	}
}

fn pname_fn(namespaces: ~[namespace], args: ~[object]) -> object
{
	if vec::len(args) == 1u
	{
		match args[0]
		{
			iri_value(iri) =>
			{
				string_value(contract_uri(namespaces, iri), ~"")
			}
			blank_value(name) =>
			{
				string_value(name, ~"")
			}
			_ =>
			{
				error_value(fmt!("rrdf:pname expected an iri_value or blank_value but was called with %?.", args[0]))
			}
		}
	}
	else
	{
		error_value(fmt!("rrdf:pname accepts 1 argument but was called with %? arguments.", vec::len(args)))
	}
}
