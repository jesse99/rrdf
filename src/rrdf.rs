#[doc = "API for the rrdf library."];
import to_str::to_str;
import core::dvec::*;
import std::map::hashmap; 
import std::time::tm;

import object::*;
import query::*;
import solution::*;
import store::*;

export subject, predicate, triple, namespace, entry, store, create_store, make_triple_blank, make_triple_str, make_triple_uri;
export object;
export solution_row, solution, selector; 
export compile;
