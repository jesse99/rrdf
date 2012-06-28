import to_str::to_str;
import core::dvec::*;
import std::map::hashmap; 
import std::time::tm;
import object::*;
import query::*;
import solution::*;
import store::*;
import types::*;

export selector;
export subject, predicate, triple, namespace, entry, store, to_str, create_store, each_triple;
export object;
export solution_row, solution; 

export compile; 
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
