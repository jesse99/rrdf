import to_str::to_str;
import core::dvec::*;
import std::map::hashmap; 
import std::time::tm;
import types::*;

// types
export subject, predicate, object, triple, namespace, binding, solution_row, solution, selector;

// this file
export to_str, create_store, store_methods, each_triple, compile,
make_bool, make_dateTime, make_double, make_int, make_lang, make_str, make_typed, make_uri,
make_triple_str, make_triple_uri;

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			{value: v, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}	{v}
			{value: v, kind: "blank", lang: ""}														{str::slice(v, 1u, str::len(v)-1u)}
			{value: v, kind: "http://www.w3.org/2001/XMLSchema#boolean", lang: ""}	{v}
			{value: v, kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""}		{#fmt["\"%s\"", v]}
			{value: v, kind: "http://www.w3.org/2001/XMLSchema#string", lang: lang}	{#fmt["\"%s\"@%s", v, lang]}
			{value: v, kind: kind, lang: ""}														{#fmt["\"%s\"^^%s", v, kind]}
			{value: v, kind: kind, lang: lang}														{#fmt["\"%s\"^^%s@%s", v, kind, lang]}	// not sure this case makes sense
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

fn make_bool(predicate: str, value: bool) -> entry
{
	if value
	{
		{predicate: predicate, object: {value: "true", kind: "http://www.w3.org/2001/XMLSchema#boolean", lang: ""}}
	}
	else
	{
		{predicate: predicate, object: {value: "false", kind: "http://www.w3.org/2001/XMLSchema#boolean", lang: ""}}
	}
}

fn make_dateTime(predicate: str, value: tm) -> entry
{
	{predicate: predicate, object: {value: value.rfc3339(), kind: "http://www.w3.org/2001/XMLSchema#dateTime", lang: ""}}
}

fn make_double(predicate: str, value: f64) -> entry
{
	{predicate: predicate, object: {value: #fmt["%?", value], kind: "http://www.w3.org/2001/XMLSchema#double", lang: ""}}
}

fn make_int(predicate: str, value: int) -> entry
{
	{predicate: predicate, object: {value: #fmt["%?", value], kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""}}
}

fn make_lang(predicate: str, value: str, lang: str) -> entry
{
	{predicate: predicate, object: {value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: lang}}
}

fn make_str(predicate: str, value: str) -> entry
{
	{predicate: predicate, object: {value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""}}
}

fn make_uri(predicate: str, value: str) -> entry
{
	{predicate: predicate, object: {value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}}
}
// TODO: add date and time

// These are for testing.
fn make_triple_blank(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: {value: #fmt["{%s}", value], kind: "blank", lang: ""}
	}
}

fn make_triple_str(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: {value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""}
	}
}

fn make_triple_uri(store: store, subject: str, predicate: str, value: str) -> triple
{
	{
		subject: expand_uri_or_blank(store.namespaces, subject), 
		predicate: expand_uri(store.namespaces, predicate), 
		object: {value: expand_uri(store.namespaces, value), kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}
	}
}

#[doc = "Note that some of the xsd datatypes should not be used: duration (which doesn't have a well defined
value space), QName, ENTITY, ID, IDREF, NOTATION, IDREFS, ENTITIES, and NMTOKENS."]
fn make_typed(predicate: str, value: str, kind: str) -> entry
{
	{predicate: predicate, object: {value: value, kind: kind, lang: ""}}
}

impl store_methods for store
{
	#[doc = "Efficient addition of triples to the store.
	
	Typically make_int, make_str, etc functions are used to create entries."]
	fn add(subject: str, entries: [entry])
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
	fn add_aggregate(subject: str, predicate: str, label: str, entries: [entry])
	{
		let blank = get_blank_name(self, label);
		self.add_triple([], {subject: subject, predicate: predicate, object: {value: blank, kind: "blank", lang: ""}});
		self.add(blank, entries);
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

// ---- Private Functions -----------------------------------------------------
fn default_namespaces() -> [namespace]
{
	[
		{prefix: "_", path: "_"},
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

fn expand_blank(name: str) -> str
{
	assert str::starts_with(name, "_:");
	
	// Curly braces are illegal IRI characters so this allows us to unambiguously
	// distinguish between IRIs and blank node references.
	ret "{" + str::slice(name, 2u, str::len(name)) + "}";
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
		expand_blank(name)
	}
	else
	{
		expand_uri(namespaces, name)
	}
}

fn expand_object(namespaces: [namespace], obj: object) -> object
{
	let kind = expand_uri_or_blank(namespaces, obj.kind);
	
	let value = 
		if kind == "blank"
		{
			expand_blank(obj.value)
		}
		else if kind == "http://www.w3.org/2001/XMLSchema#anyURI"
		{
			expand_uri(namespaces, obj.value)
		}
		else
		{
			obj.value
		};
	
	{value: value, kind: kind, lang: obj.lang}
}

fn expand_entry(namespaces: [namespace], entry: entry) -> entry
{
	{predicate: expand_uri(namespaces, entry.predicate), object: expand_object(namespaces, entry.object)}
}
