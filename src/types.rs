#[doc = "The types exported by the rrdf library. The most important of which are store and triple."];
import core::dvec::*;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)
or a blank node (_1)."]
type subject = str;

#[doc = "An internationalized URI with an optional fragment identifier (http://www.w3.org/2001/XMLSchema#date)."]
type predicate = str;

#[doc = "Used internally be the store class to store prefixed names."]
type qname = {nindex: uint, name: str};				// nindex is into namespaces

#[doc = "Value component of a triple.

* reference identifies a subject. \"_:foo\" references a blank node.
* qref is used internally by the store class to identify a subject.
* typed_literal is an arbitrary lexical value along with an IRI for its type. Most common types are 
   xsd:boolean, xsd:double, xsd:anyURI, xsd:string, xsd:integer, and xsd:positiveInteger, 
   see http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"XML Schema\" for more.
* plain_literal is a string along with a language tag (e.g. \"en-us\") See http://tools.ietf.org/html/bcp47 \"Tags for Identifying Languages\".
* xml is embedded xml, see: http://www.w3.org/TR/2011/WD-rdf11-concepts-20110830/#dfn-rdf-xmlliteral."]
enum object
{
	reference(subject),
	qref(qname),
	typed_literal(str, str),
	plain_literal(str, str),
	xml(str),
}

#[doc = "A relationship between a subject and an object.

* subject identifies a resource, usually via an IRI.
* predicate is an IRI describing the relationship. Also known as a property.
* object is a IRI, literal, or blank node containing the value for the relationship.

Here is a psuedo-code example:

('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')"]
type triple = {subject: subject, predicate: predicate, object: object};

#[doc = "Used internally by the store record."]
type entry = {predicate: qname, object: object};

#[doc = "Name of a namespace plus the IRI it expands to."]
type namespace = {prefix: str, path: str};

#[doc = "Stores triples in a more or less efficient format."]
type store = {
	namespaces: [namespace],					// 0 == "" (no namespace), 1 == "_" (blank)
	subjects: hashmap<qname, dvec<entry>>
};

