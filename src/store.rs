//! The type which stores triples.
use core::dvec::*;
use std::map::*;
use object::*;
use solution::*;

export Subject, Predicate, Triple, Namespace, Entry, ExtensionFn, Store, make_triple_blank, 
	make_triple_str, make_triple_uri, StoreTrait, store_methods, to_str, base_iter, get_blank_name, contract_uri;
export expand_uri;			// this should be internal
export Expr, Pattern, Variable, Constant, TriplePattern, Algebra, Basic, Group, Optional, Bind, Filter,
	ConstantExpr, VariableExpr, CallExpr, ExtensionExpr, QueryContext, object_to_str, get_object;

// --------------------------------------------------------------------------------------
// TODO: should be in expression.rs (see rust bug 3352)
enum Expr
{
	ConstantExpr(Object),
	VariableExpr(~str),
	CallExpr(~str, ~[@Expr]),			// function name + arguments
	ExtensionExpr(~str, ~[@Expr])	// function name + arguments
}

// --------------------------------------------------------------------------------------
// TODO: should be in query.rs (see rust bug 3352)
enum Pattern
{
	Variable(~str),
	Constant(Object)
}

type TriplePattern = {subject: Pattern, predicate: Pattern, object: Pattern};

enum Algebra
{
	Basic(TriplePattern),
	Group(~[@Algebra]),
	Optional(@Algebra),
	Bind(Expr, ~str),
	Filter(Expr)
}

type QueryContext =
{
	namespaces: @~[Namespace],
	extensions: @hashmap<@~str, ExtensionFn>,
	algebra: Algebra,
	order_by: ~[Expr],
	distinct: bool,
	limit: Option<uint>,
	rng: rand::Rng,		// for RAND
	timestamp: Tm		// for NOW
};

// --------------------------------------------------------------------------------------
// TODO: should be in object.rs (see rust bug 3352)
fn object_to_str(store: &Store, obj: Object) -> ~str
{
	match obj
	{
		TypedValue(value, kind) =>
		{
			fmt!("\"%s^^\"%s", value, contract_uri(store.namespaces, kind))
		}
		IriValue(iri) =>
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
fn get_object(row: SolutionRow, name: ~str) -> Object
{
	match row.search(name)
	{
		option::Some(value) =>
		{
			value
		}
		option::None =>
		{
			UnboundValue(name)
		}
	}
}

// --------------------------------------------------------------------------------------
/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
/// or a blank node (_1).
type Subject = ~str;

/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date).
type Predicate = ~str;

/// A relationship between a subject and an object.
/// 
/// * subject identifies a resource, usually via an IRI.
/// * predicate is an IRI describing the relationship. Also known as a property.
/// * object is a IRI, literal, or blank node containing the value for the relationship.
/// 
/// Here is a psuedo-code example:
/// 
/// ('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')
type Triple = {subject: Subject, predicate: Predicate, object: Object};

/// Predicate and object associated with a subject.
type Entry = {predicate: ~str, object: Object};

/// SPARQL extension function.
type ExtensionFn = fn@ (namespaces: &~[Namespace], args: &~[Object]) -> Object;

/// Stores triples in a more or less efficient format.
///
/// Note that these are not intended to be copied.
struct Store : ToStr
{
	namespaces: ~[Namespace];
	subjects: hashmap<@~str, @DVec<Entry>>;
	extensions: hashmap<@~str, ExtensionFn>;
	mut next_blank: int;
	
	// TODO: add a drop method (to make Stores non-copyable)
	
	fn to_str() -> ~str
	{
		let mut result = ~"";
		
		for self.subjects.each()
		|subject, entries|
		{
			for (*entries).eachi()
			|i, entry|
			{
				result += fmt!("%?: <%s>  <%s>  %s}\n", i, *subject, entry.predicate, entry.object.to_str());
			}
		};
		
		return result;
	}
}

/// Initializes a store object.
/// 
/// xsd, rdf, rdfs, and owl namespaces are automatically added. An rrdf:pname extension is
/// automatically added which converts an IriValue to a StringValue using namespaces (or
/// simply stringifies it if none of the namespaces paths match).
fn Store(namespaces: ~[Namespace], extensions: &hashmap<@~str, ExtensionFn>) -> Store
{
	let store = Store {
		namespaces: default_namespaces() + namespaces,
		subjects: std::map::box_str_hash(),
		extensions: copy *extensions,
		next_blank: 0,
	};
	
	store.extensions.insert(@~"rrdf:pname", pname_fn);
	store
}

fn get_blank_name(store: &Store, prefix: ~str) -> ~str
{
	let suffix = store.next_blank;
	store.next_blank += 1;
	
	fmt!("_:%s-%?", prefix, suffix)
}

/// Returns either the iri or the prefixed version of the iri.
fn contract_uri(namespaces: ~[Namespace], iri: ~str) -> ~str
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

trait StoreTrait
{
	/// Efficient addition of triples to the store.
	/// 
	/// Typically create_int, create_str, etc functions are used to create objects.
	fn add(subject: ~str, entries: ~[(~str, Object)]);
	
	/// Relatively inefficient addition of triples to the store.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn add_triple(namespaces: ~[Namespace], triple: Triple);
	
	/// Adds a subject statement referencing a new blank node.
	/// 
	/// Label is an arbitrary string useful for debugging. Returns the name of the blank node.
	fn add_aggregate(subject: ~str, predicate: ~str, label: ~str, entries: ~[(~str, Object)]) -> ~str;
	
	/// Adds statements representing a choice between alternatives.
	fn add_alt(subject: ~str, values: ~[Object]);
	
	/// Adds statements representing an unordered set of (possibly duplicate) values.
	fn add_bag(subject: ~str, values: ~[Object]);
	
	/// Adds statements representing an arbitrary open container using 1-based integral keys.
	fn add_container(subject: ~str, kind: ~str, values: ~[Object]);
	
	/// Adds a fixed size list of (possibly duplicate) items.
	fn add_list(subject: ~str, predicate: ~str, values: ~[Object]);
	
	/// Adds a statement about a statement.
	/// 
	/// Often used for meta-data, e.g. a timestamp stating when a statement was added to the store.
	fn add_reify(subject: ~str, predicate: ~str, value: Object);
	
	/// Adds statements representing an ordered set of (possibly duplicate) values.
	fn add_seq(subject: ~str, values: ~[Object]);
	
	/// Removes all triples from the store.
	fn clear();
	
	/// Returns the first matching object, or option::none.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_object(subject: ~str, predicate: ~str) -> option::Option<Object>;
	
	/// Returns all matching objects.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_objects(subject: ~str, predicate: ~str) -> ~[Object];
	
	/// Replaces the object of an existing triple or adds a new triple.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn replace_triple(namespaces: ~[Namespace], triple: Triple);
}

impl  &Store : StoreTrait 
{
	fn add(subject: ~str, entries: ~[(~str, Object)])
	{
		if vec::is_not_empty(entries)
		{
			let subject = expand_uri_or_blank(self.namespaces, subject);
			let entries = vec::map(entries, |e| {expand_entry(self.namespaces, e)});
			match self.subjects.find(@subject)
			{
				option::Some(list) =>
				{
					(*list).push_all(entries);
				}
				option::None =>
				{
					let list = @DVec();
					self.subjects.insert(@subject, list);
					(*list).push_all(entries);
				}
			}
		}
	}
	
	fn add_triple(namespaces: ~[Namespace], triple: Triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(@subject)
		{
			option::Some(entries) =>
			{
				(*entries).push(entry);
			}
			option::None =>
			{
				self.subjects.insert(@subject, @dvec::from_vec(~[mut entry]));
			}
		}
	}
	
	fn add_aggregate(subject: ~str, predicate: ~str, label: ~str, entries: ~[(~str, Object)]) -> ~str
	{
		let blank = get_blank_name(self, label);
		self.add_triple(~[], {subject: subject, predicate: predicate, object: BlankValue(blank)});
		self.add(blank, entries);
		return blank;
	}
	
	fn add_alt(subject: ~str, values: ~[Object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt", values);
	}
	
	fn add_bag(subject: ~str, values: ~[Object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag", values);
	}
	
	fn add_container(subject: ~str, kind: ~str, values: ~[Object])
	{
		let blank = get_blank_name(self, after(subject, ':') + "-items");
		self.add_triple(~[], {subject: subject, predicate: kind, object: BlankValue(blank)});
		
		let predicates = do iter::map_to_vec(vec::len(values)) |i: uint| {fmt!("http://www.w3.org/1999/02/22-rdf-syntax-ns#_%?", i+1u)};
		self.add(blank, vec::zip(predicates, values));
	}
	
	fn add_list(subject: ~str, predicate: ~str, values: ~[Object])
	{
		let prefix = after(predicate, ':');
		let mut blank = get_blank_name(self, prefix);
		self.add_triple(~[], {subject: subject, predicate: predicate, object: BlankValue(blank)});
		for vec::each(values)
		|value|
		{
			let next = get_blank_name(self, prefix);
			self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#first", object: value});
			self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: BlankValue(next)});
			blank = next;
		};
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: IriValue(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#nil")});
	}
	
	fn add_reify(subject: ~str, predicate: ~str, value: Object)
	{
		let mut blank = get_blank_name(self, after(predicate, ':'));
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#type", object: IriValue(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement")});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#subject", object: IriValue(subject)});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate", object: IriValue(predicate)});
		self.add_triple(~[], {subject: blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#object", object: value});
	}
	
	fn add_seq(subject: ~str, values: ~[Object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Seq", values);
	}
	
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
	
	fn find_object(subject: ~str, predicate: ~str) -> option::Option<Object>
	{
		let subject = expand_uri_or_blank(self.namespaces, subject);
		let predicate = expand_uri(self.namespaces, predicate);
		
		match self.subjects.find(@subject)
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
	
	fn find_objects(subject: ~str, predicate: ~str) -> ~[Object]
	{
		let subject = expand_uri_or_blank(self.namespaces, subject);
		let predicate = expand_uri(self.namespaces, predicate);
		
		match self.subjects.find(@subject)
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
	
	fn replace_triple(namespaces: ~[Namespace], triple: Triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(@subject)
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
				self.subjects.insert(@subject, @dvec::from_vec(~[mut entry]));
			}
		}
	}
}

impl Store : BaseIter<Triple>
{
	/// Calls the blk for each triple in the store (in an undefined order).
	pure fn each(blk: fn(Triple) -> bool)
	{
		unchecked		// TODO: remove once bug 3372 is fixed
		{
			for self.subjects.each()
			|subject, entries|
			{
				for (*entries).each()
				|entry|
				{
					let triple = {subject: *subject, predicate: entry.predicate, object: entry.object};
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

impl  Triple : ToStr 
{
	fn to_str() -> ~str
	{
		fmt!("{%s, %s, %s}", self.subject, self.predicate, self.object.to_str())
	}
}

// ---- Private Functions -----------------------------------------------------
fn default_namespaces() -> ~[Namespace]
{
	~[
		{prefix: ~"_", path: ~"_:"},
		{prefix: ~"xsd", path: ~"http://www.w3.org/2001/XMLSchema#"},
		{prefix: ~"rdf", path: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#"},
		{prefix: ~"rdfs", path: ~"http://www.w3.org/2000/01/rdf-schema#"},
		{prefix: ~"owl", path: ~"http://www.w3.org/2002/07/owl#"}
	]
}

fn expand_uri(namespaces: ~[Namespace], name: ~str) -> ~str
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

fn expand_uri_or_blank(namespaces: ~[Namespace], name: ~str) -> ~str
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

fn expand_object(namespaces: ~[Namespace], obj: Object) -> Object
{
	match obj
	{
		TypedValue(value, kind) =>
		{
			TypedValue(value, expand_uri(namespaces, kind))
		}
		IriValue(value) =>
		{
			IriValue(expand_uri(namespaces, value))
		}
		BlankValue(value) =>
		{
			BlankValue(expand_uri(namespaces, value))
		}
		_ =>
		{
			obj
		}
	}
}

fn expand_entry(namespaces: ~[Namespace], entry: (~str, Object)) -> Entry
{
	{predicate: expand_uri(namespaces, entry.first()), object: expand_object(namespaces, entry.second())}
}

fn make_triple_blank(store: Store, subject: ~str, predicate: ~str, value: ~str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: BlankValue(fmt!("_:%s", value))
	}
}

fn make_triple_str(store: Store, subject: ~str, predicate: ~str, value: ~str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: StringValue(value, ~"")
	}
}

fn make_triple_uri(store: Store, subject: ~str, predicate: ~str, value: ~str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: IriValue(expand_uri(store.namespaces, value))
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

fn pname_fn(namespaces: &~[Namespace], args: &~[Object]) -> Object
{
	if vec::len(*args) == 1u
	{
		match args[0]
		{
			IriValue(iri) =>
			{
				StringValue(contract_uri(*namespaces, iri), ~"")
			}
			BlankValue(name) =>
			{
				StringValue(name, ~"")
			}
			_ =>
			{
				ErrorValue(fmt!("rrdf:pname expected an IriValue or BlankValue but was called with %?.", args[0]))
			}
		}
	}
	else
	{
		ErrorValue(fmt!("rrdf:pname accepts 1 argument but was called with %? arguments.", vec::len(*args)))
	}
}
