#[doc = "The type which stores triples."];
import core::dvec::*;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
or a blank node (_1)."]
type subject = str;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)."]
type predicate = str;

#[doc = "A relationship between a subject and an object.

* subject identifies a resource, usually via an IRI.
* predicate is an IRI describing the relationship. Also known as a property.
* object is a IRI, literal, or blank node containing the value for the relationship.

Here is a psuedo-code example:

('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')"]
type triple = {subject: subject, predicate: predicate, object: object};

#[doc = "Name of a namespace plus the IRI it expands to."]
type namespace = {prefix: str, path: str};

#[doc = "Predicate and object associated with a subject."]
type entry = {predicate: str, object: object};

#[doc = "Stores triples in a more or less efficient format."]
type store = {
	namespaces: [namespace],
	subjects: hashmap<str, @dvec<entry>>,
	mut next_blank: uint
};

impl of to_str for triple
{
	fn to_str() -> str
	{
		#fmt["{%s, %s, %s}", self.subject, self.predicate, self.object.to_str()]
	}
}

impl of to_str for store
{
	fn to_str() -> str
	{
		let mut result = "";
		
		for self.subjects.each()
		{|subject, entries|
			for (*entries).eachi()
			{|i, entry|
				result += #fmt["%?: <%s>  <%s>  %s}\n", i, subject, entry.predicate, entry.object.to_str()];
			}
		};
		
		ret result;
	}
}

#[doc = "Initializes a store object.

xsd, rdf, rdfs, and owl namespaces are automatically added."]
fn create_store(namespaces: [namespace]) -> store
{
	{
		namespaces: default_namespaces() + namespaces,
		subjects: std::map::str_hash(),
		mut next_blank: 0u
	}
}

impl store_methods for store
{
	#[doc = "Efficient addition of triples to the store.
	
	Typically create_int, create_str, etc functions are used to create objects."]
	fn add(subject: str, entries: [(str, object)])
	{
		if vec::is_not_empty(entries)
		{
			let subject = expand_uri_or_blank(self.namespaces, subject);
			let entries = vec::map(entries, {|e| expand_entry(self.namespaces, e)});
			alt self.subjects.find(subject)
			{
				option::some(list)
				{
					(*list).push_all(entries);
				}
				option::none
				{
					let list = @dvec();
					self.subjects.insert(subject, list);
					(*list).push_all(entries);
				}
			}
		}
	}
	
	#[doc = "Relatively inefficient addition of triples to the store.
	
	Qualified names may use the namespaces associated with the store and the supplied namespaces."]
	fn add_triple(namespaces: [namespace], triple: triple)
	{
		let namespaces = self.namespaces + namespaces;
		
		let subject = expand_uri_or_blank(namespaces, triple.subject);
		let predicate = expand_uri(namespaces, triple.predicate);
		let entry = {predicate: predicate, object: expand_object(namespaces, triple.object)};
		
		alt self.subjects.find(subject)
		{
			option::some(entries)
			{
				(*entries).push(entry);
			}
			option::none
			{
				self.subjects.insert(subject, @dvec::from_vec([mut entry]));
			}
		}
	}
	
	#[doc = "Adds a subject statement referencing a new blank node.
	
	Label is an arbitrary string useful for debugging."]
	fn add_aggregate(subject: str, predicate: str, label: str, entries: [(str, object)])
	{
		let blank = get_blank_name(self, label);
		self.add_triple([], {subject: subject, predicate: predicate, object: blank_value(blank)});
		self.add(blank, entries);
	}
	
	#[doc = "Adds statements representing a choice between alternatives."]
	fn add_alt(subject: str, values: [object])
	{
		self.add_container(subject, "http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt", values);
	}
	
	#[doc = "Adds statements representing an unordered set of (possibly duplicate) values."]
	fn add_bag(subject: str, values: [object])
	{
		self.add_container(subject, "http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag", values);
	}
	
	#[doc = "Adds statements representing an arbitrary open container using 1-based integral keys."]
	fn add_container(subject: str, kind: str, values: [object])
	{
		let blank = get_blank_name(self, after(subject, ':') + "-items");
		self.add_triple([], {subject: subject, predicate: kind, object: blank_value(blank)});
		
		let predicates = iter::map_to_vec(vec::len(values)) {|i: uint| #fmt["http://www.w3.org/1999/02/22-rdf-syntax-ns#_%?", i+1u]};
		self.add(blank, vec::zip(predicates, values));
	}
	
	#[doc = "Adds a fixed size list of (possibly duplicate) items."]
	fn add_list(subject: str, predicate: str, values: [object])
	{
		let prefix = after(predicate, ':');
		let mut blank = get_blank_name(self, prefix);
		self.add_triple([], {subject: subject, predicate: predicate, object: blank_value(blank)});
		for vec::each(values)
		{|value|
			let next = get_blank_name(self, prefix);
			self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#first", object: value});
			self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: blank_value(next)});
			blank = next;
		};
		self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest", object: iri_value("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil")});
	}
	
	#[doc = "Adds a statement about a statement.
	
	Often used for meta-data, e.g. a timestamp stating when a statement was added to the store."]
	fn add_reify(subject: str, predicate: str, value: object)
	{
		let mut blank = get_blank_name(self, after(predicate, ':'));
		self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", object: iri_value("http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement")});
		self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject", object: iri_value(subject)});
		self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate", object: iri_value(predicate)});
		self.add_triple([], {subject: blank, predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#object", object: value});
	}
	
	#[doc = "Adds statements representing an ordered set of (possibly duplicate) values."]
	fn add_seq(subject: str, values: [object])
	{
		self.add_container(subject, "http://www.w3.org/1999/02/22-rdf-syntax-ns#Seq", values);
	}
}

// TODO: may want to make these methods
#[doc = "Calls the callback for each triple in the store (in an undefined order)."]
fn each_triple(store: store, callback: fn (triple) -> bool) unsafe
{
	for store.subjects.each()
	{|subject, entries|
		for (*entries).each()
		{|entry|
			let triple = {subject: subject, predicate: entry.predicate, object: entry.object};
			if !callback(triple)
			{
				ret;
			}
		}
	};
}

// ---- Private Functions -----------------------------------------------------
fn default_namespaces() -> [namespace]
{
	[
		{prefix: "_", path: "_:"},
		{prefix: "xsd", path: "http://www.w3.org/2001/XMLSchema#"},
		{prefix: "rdf", path: "http://www.w3.org/1999/02/22-rdf-syntax-ns#"},
		{prefix: "rdfs", path: "http://www.w3.org/2000/01/rdf-schema#"},
		{prefix: "owl", path: "http://www.w3.org/2002/07/owl#"}
	]
}

fn get_blank_name(store: store, prefix: str) -> str
{
	let name = #fmt["_:%s-%?", prefix, copy(store.next_blank)];
	store.next_blank += 1u;
	ret name;
}

fn expand_uri(namespaces: [namespace], name: str) -> str
{
	// TODO: need to % escape bogus characters (after converting to utf-8)
	for vec::each(namespaces)
	{|namespace|
		if str::starts_with(name, namespace.prefix + ":")
		{
			ret namespace.path + str::slice(name, str::len(namespace.prefix) + 1u, str::len(name));
		}
	};
	ret name;
}

fn expand_uri_or_blank(namespaces: [namespace], name: str) -> str
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

fn expand_object(namespaces: [namespace], obj: object) -> object
{
	alt obj
	{
		typed_value(value, kind)
		{
			typed_value(value, expand_uri(namespaces, kind))
		}
		iri_value(value)
		{
			iri_value(expand_uri(namespaces, value))
		}
		blank_value(value)
		{
			blank_value(expand_uri(namespaces, value))
		}
		_
		{
			obj
		}
	}
}

fn expand_entry(namespaces: [namespace], entry: (str, object)) -> entry
{
	{predicate: expand_uri(namespaces, tuple::first(entry)), object: expand_object(namespaces, tuple::second(entry))}
}

fn make_triple_blank(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: blank_value(#fmt["_:%s", value])
	}
}

fn make_triple_str(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: string_value(value, "")
	}
}

fn make_triple_uri(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: iri_value(expand_uri(store.namespaces, value))
	}
}

impl of iter::base_iter<uint> for uint
{
	fn each(blk: fn(&&uint) -> bool)
	{
		let mut i = 0u;
		while i < self
		{
			if (!blk(i))
			{
				ret;
			}
			i += 1u;
		}
	}
	
	fn size_hint() -> option<uint>
	{
		option::some(self)
	}
}

fn after(text: str, ch: char) -> str
{
	alt str::rfind_char(text, ch)
	{
		option::some(i)
		{
			str::slice(text, i+1u, str::len(text))
		}
		option::none
		{
			text
		}
	}
}
