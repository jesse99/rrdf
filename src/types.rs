#[doc = "The types exported by the rrdf library. The most important of which are store, triple, solution, and selector."];
import core::dvec::*;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
or a blank node (_1)."]
type subject = str;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)."]
type predicate = str;

#[doc = "Value component of a triple.

* value is an abitrary lexical value.
* kind is a prefixed name or an IRI indicating the value's type. Common types include 
   xsd:boolean, xsd:string, xsd:anyURI, xsd:double, xsd:integer, and xsd:positiveInteger, 
   see http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"XML Schema\" for more.
* lang is either empty for an undefined language or a tag like  \"en-us\". See http://tools.ietf.org/html/bcp47 \"Tags for Identifying Languages\" for more."]
type object = {value: str, kind: str, lang: str};

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

#[doc = "Value bound to a variable in a SPARQL query (e.g. ?name)."]
type binding = {name: str, value: object};

#[doc = "Names appear in the same order as the variables in the SELECT clause."]
type solution_row = [binding];

#[doc = "Result of a SPARQL query."]
type solution = [solution_row];

#[doc = "The function returned by compile and invoked to execute a SPARQL query.

Returns a solution or a 'runtime' error."]
type selector = fn@ (store) -> result::result<solution, str>;

