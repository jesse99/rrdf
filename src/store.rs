//! The type which stores triples.

// --------------------------------------------------------------------------------------
/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
/// or a blank node (_1).
pub type Subject = ~str;

/// An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date).
pub type Predicate = ~str;

/// A relationship between a subject and an object.
/// 
/// * subject identifies a resource, usually via an IRI.
/// * predicate is an IRI describing the relationship. Also known as a property.
/// * object is a IRI, literal, or blank node containing the value for the relationship.
/// 
/// Here is a psuedo-code example:
/// 
/// ('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')
pub type Triple = {subject: Subject, predicate: Predicate, object: @Object};

/// Predicate and object associated with a subject.
pub type Entry = {predicate: ~str, object: @Object};

// TODO: This is hopefully temporary: at some point rust should again be able to compare enums without assistence.
pub impl Entry : cmp::Eq
{
	pure fn eq(&self, other: &Entry) -> bool
	{
		self.predicate == other.predicate && self.object == other.object
	}
	
	pure fn ne(&self, other: &Entry) -> bool
	{
		!self.eq(other)
	}
}

/// SPARQL extension function.
pub type ExtensionFn = pure fn@ (namespaces: &[Namespace], args: &[@Object]) -> @Object;

/// Stores triples in a more or less efficient format.
///
/// Note that these are not intended to be copied.
pub struct Store
{
	pub namespaces: ~[Namespace],
	pub subjects: HashMap<@~str, @DVec<Entry>>,
	pub extensions: HashMap<@~str, ExtensionFn>,
	pub mut next_blank: int,
	
	drop {}
}

/// Initializes a store object.
/// 
/// xsd, rdf, rdfs, and owl namespaces are automatically added. An rrdf:pname extension is
/// automatically added which converts an IriValue to a StringValue using namespaces (or
/// simply stringifies it if none of the namespaces paths match).
pub fn Store(namespaces: &[Namespace], extensions: &HashMap<@~str, ExtensionFn>) -> Store
{
	let store = Store {
		namespaces: default_namespaces() + namespaces,
		subjects: HashMap(),
		extensions: copy *extensions,
		next_blank: 0,
	};
	
	store.extensions.insert(@~"rrdf:pname", pname_fn);
	store
}

pub fn get_blank_name(store: &Store, prefix: &str) -> ~str
{
	let suffix = store.next_blank;
	store.next_blank += 1;
	
	fmt!("_:%s-%?", prefix, suffix)
}

/// Returns either the iri or the prefixed version of the iri.
pub pure fn contract_uri(namespaces: &[Namespace], iri: &str) -> ~str
{
	match vec::find(namespaces, |n| {str::starts_with(iri, n.path)})
	{
		option::Some(ref ns) =>
		{
			fmt!("%s:%s", ns.prefix, str::slice(iri, str::len(ns.path), str::len(iri)))
		}
		option::None =>
		{
			iri.to_owned()
		}
	}
}

pub trait StoreTrait
{
	/// Efficient addition of triples to the store.
	/// 
	/// Typically create_int, create_str, etc functions are used to create objects.
	fn add(subject: &str, entries: &[(~str, @Object)]);
	
	/// Relatively inefficient addition of triples to the store.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn add_triple(namespaces: &[Namespace], triple: Triple);
	
	/// Adds a subject statement referencing a new blank node.
	/// 
	/// Label is an arbitrary string useful for debugging. Returns the name of the blank node.
	fn add_aggregate(subject: &str, predicate: &str, label: &str, entries: &[(~str, @Object)]) -> ~str;
	
	/// Adds statements representing a choice between alternatives.
	fn add_alt(subject: &str, values: &[@Object]);
	
	/// Adds statements representing an unordered set of (possibly duplicate) values.
	fn add_bag(subject: &str, values: &[@Object]);
	
	/// Adds statements representing an arbitrary open container using 1-based integral keys.
	fn add_container(subject: &str, kind: &str, values: &[@Object]);
	
	/// Adds a fixed size list of (possibly duplicate) items.
	fn add_list(subject: &str, predicate: &str, values: &[@Object]);
	
	/// Adds a statement about a statement.
	/// 
	/// Often used for meta-data, e.g. a timestamp stating when a statement was added to the store.
	fn add_reify(subject: &str, predicate: &str, value: @Object);
	
	/// Adds statements representing an ordered set of (possibly duplicate) values.
	fn add_seq(subject: &str, values: &[@Object]);
	
	/// Removes all triples from the store.
	fn clear();
	
	/// Returns the first matching object, or option::none.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_object(subject: &str, predicate: &str) -> option::Option<@Object>;
	
	/// Returns all matching objects.
	/// 
	/// Qualified names may use the namespaces associated with the store.
	fn find_objects(subject: &str, predicate: &str) -> ~[@Object];
	
	/// Replaces the object of an existing triple or adds a new triple.
	/// 
	/// Qualified names may use the namespaces associated with the store and the supplied namespaces.
	fn replace_triple(namespaces: &[Namespace], triple: Triple);
}

pub impl  &Store : StoreTrait 
{
	fn add(subject: &str, entries: &[(~str, @Object)])
	{
		if vec::is_not_empty(entries)
		{
			let subject = expand_uri_or_blank(self.namespaces, subject);
			let entries = vec::map(entries, |e| {expand_entry(self.namespaces, e)});
			match self.subjects.find(@copy subject)
			{
				option::Some(list) =>
				{
					list.push_all(entries);
				}
				option::None =>
				{
					let list = @DVec();
					self.subjects.insert(@subject, list);
					list.push_all(entries);
				}
			}
		}
	}
	
	fn add_triple(namespaces: &[Namespace], triple: Triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(@copy subject)
		{
			option::Some(entries) =>
			{
				entries.push(entry);
			}
			option::None =>
			{
				self.subjects.insert(@subject, @dvec::from_vec(~[entry]));
			}
		}
	}
	
	fn add_aggregate(subject: &str, predicate: &str, label: &str, entries: &[(~str, @Object)]) -> ~str
	{
		let blank = get_blank_name(self, label);
		self.add_triple(~[], {subject: subject.to_owned(), predicate: predicate.to_owned(), object: @BlankValue(blank.to_owned())});
		self.add(blank, entries);
		return blank;
	}
	
	fn add_alt(subject: &str, values: &[@Object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt", values);
	}
	
	fn add_bag(subject: &str, values: &[@Object])
	{
		self.add_container(subject, ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag", values);
	}
	
	fn add_container(subject: &str, kind: &str, values: &[@Object])
	{
		let blank = get_blank_name(self, after(subject, ':') + "-items");
		self.add_triple(~[], {subject: subject.to_owned(), predicate: kind.to_owned(), object: @BlankValue(blank.to_owned())});
		
		let predicates = do iter::map_to_vec(&vec::len(values)) |i: &uint| {fmt!("http://www.w3.org/1999/02/22-rdf-syntax-ns#_%?", *i+1u)};
		self.add(blank, vec::zip(predicates, vec::from_slice(values)));
	}
	
	fn add_list(subject: &str, predicate: &str, values: &[@Object])
	{
		let prefix = after(predicate, ':');
		let mut blank = get_blank_name(self, prefix);
		self.add_triple(~[], {subject: subject.to_owned(), predicate: predicate.to_owned(), object: @BlankValue(blank.to_owned())});
		for vec::each(values) |value|
		{
			let next = get_blank_name(self, prefix);
			self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#first", object: *value});
			self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: @BlankValue(copy next)});
			blank = next;
		};
		self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: @IriValue(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#nil")});
	}
	
	fn add_reify(subject: &str, predicate: &str, value: @Object)
	{
		let mut blank = get_blank_name(self, after(predicate, ':'));
		self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#type", object: @IriValue(~"http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement")});
		self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#subject", object: @IriValue(subject.to_owned())});
		self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate", object: @IriValue(predicate.to_owned())});
		self.add_triple(~[], {subject: copy blank, predicate: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#object", object: value});
	}
	
	fn add_seq(subject: &str, values: &[@Object])
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
			vec::push(&mut keys, key);
		};
		
		for vec::each(keys)
		|key|
		{
			self.subjects.remove(*key);
		};
	}
	
	fn find_object(subject: &str, predicate: &str) -> option::Option<@Object>
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
						option::Some(copy (*entries)[index].object)
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
	
	fn find_objects(subject: &str, predicate: &str) -> ~[@Object]
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
						option::Some(copy entry.object)
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
	
	fn replace_triple(namespaces: &[Namespace], triple: Triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: copy predicate, object: expand_object(namespaces, triple.object)};
		
		match self.subjects.find(@copy subject)
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
				self.subjects.insert(@subject, @dvec::from_vec(~[entry]));
			}
		}
	}
}

pub impl &Store : ToStr
{
	pure fn to_str() -> ~str
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

pub impl Store : BaseIter<Triple>
{
	/// Calls the blk for each triple in the store (in an undefined order).
	pure fn each(&self, blk: fn(v: &Triple) -> bool)
	{
		unsafe		// TODO: remove once bug 3372 is fixed
		{
			for self.subjects.each() |subject, entries|
			{
				for (*entries).each() |entry|
				{
					let triple = {subject: copy *subject, predicate: copy entry.predicate, object: copy entry.object};
					if !blk(&triple)
					{
						return;
					}
				}
			};
		}
	}
	
	pure fn size_hint(&self) -> Option<uint>
	{
		unsafe
		{
			option::Some(self.subjects.size())
		}
	}
}

pub impl  Triple : ToStr
{
	pure fn to_str() -> ~str
	{
		fmt!("{%s, %s, %s}", self.subject, self.predicate, self.object.to_str())
	}
}

// ---- Private Functions -----------------------------------------------------
priv fn default_namespaces() -> ~[Namespace]
{
	~[
		Namespace {prefix: ~"_", path: ~"_:"},
		Namespace {prefix: ~"xsd", path: ~"http://www.w3.org/2001/XMLSchema#"},
		Namespace {prefix: ~"rdf", path: ~"http://www.w3.org/1999/02/22-rdf-syntax-ns#"},
		Namespace {prefix: ~"rdfs", path: ~"http://www.w3.org/2000/01/rdf-schema#"},
		Namespace {prefix: ~"owl", path: ~"http://www.w3.org/2002/07/owl#"}
	]
}

priv fn expand_uri(namespaces: &[Namespace], name: &str) -> ~str
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
	return name.to_owned();
}

priv fn expand_uri_or_blank(namespaces: &[Namespace], name: &str) -> ~str
{
	if str::starts_with(name, "_:")
	{
		name.to_owned()
	}
	else
	{
		expand_uri(namespaces, name)
	}
}

priv fn expand_object(namespaces: &[Namespace], obj: @Object) -> @Object
{
	match *obj
	{
		TypedValue(copy value, ref kind) =>
		{
			@TypedValue(value, expand_uri(namespaces, *kind))
		}
		IriValue(ref value) =>
		{
			@IriValue(expand_uri(namespaces, *value))
		}
		BlankValue(ref value) =>
		{
			@BlankValue(expand_uri(namespaces, *value))
		}
		_ =>
		{
			obj
		}
	}
}

priv fn expand_entry(namespaces: &[Namespace], entry: &(~str, @Object)) -> Entry
{
	{predicate: expand_uri(namespaces, entry.first()), object: expand_object(namespaces, entry.second())}
}

priv fn make_triple_blank(store: &Store, subject: &str, predicate: &str, value: &str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: @BlankValue(fmt!("_:%s", value))
	}
}

priv fn make_triple_str(store: &Store, subject: &str, predicate: &str, value: &str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: @StringValue(value.to_owned(), ~"")
	}
}

priv fn make_triple_uri(store: &Store, subject: &str, predicate: &str, value: &str) -> Triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: @IriValue(expand_uri(store.namespaces, value))
	}
}

impl uint : BaseIter<uint>
{
	pure fn each(&self, blk: fn(v: &uint) -> bool)
	{
		let mut i = 0u;
		while i < *self
		{
			if (!blk(&i))
			{
				return;
			}
			i += 1u;
		}
	}
	
	pure fn size_hint(&self) -> Option<uint>
	{
		option::Some(*self)
	}
}

priv fn after(text: &str, ch: char) -> ~str
{
	match str::rfind_char(text, ch)
	{
		option::Some(i) =>
		{
			str::slice(text, i+1u, str::len(text))
		}
		option::None =>
		{
			text.to_owned()
		}
	}
}

priv pure fn pname_fn(namespaces: &[Namespace], args: &[@Object]) -> @Object
{
	if args.len() == 1
	{
		match *(args[0])
		{
			IriValue(ref iri) =>
			{
				@StringValue(contract_uri(namespaces, *iri), ~"")
			}
			BlankValue(copy name) =>
			{
				@StringValue(name, ~"")
			}
			_ =>
			{
				@ErrorValue(fmt!("rrdf:pname expected an IriValue or BlankValue but was called with %?.", args[0]))
			}
		}
	}
	else
	{
		@ErrorValue(fmt!("rrdf:pname accepts 1 argument but was called with %? arguments.", args.len()))
	}
}
