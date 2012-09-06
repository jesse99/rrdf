//! API for the rrdf library.
use to_str::to_str;
use core::dvec::*;
use std::map::hashmap; 
use std::time::tm;

// This is a convenience for internal modules.
use object::*;
use query::*;
use solution::*;
use store::*;

// This is the public API. Clients should only use the items exported here.
// TODO: Hopefully we can clean this up a lot when exporting works a bit better.
use literal_to_object = object::literal_to_object; export literal_to_object;
use Object = object::Object; export Object;
use object_methods = object::object_methods; export object_methods;

use BlankValue = object::BlankValue; export BlankValue;
use BoolValue = object::BoolValue; export BoolValue;
use DateTimeValue = object::DateTimeValue; export DateTimeValue;
use ErrorValue = object::ErrorValue; export ErrorValue;
use FloatValue = object::FloatValue; export FloatValue;
use IntValue = object::IntValue; export IntValue;
use InvalidValue = object::InvalidValue; export InvalidValue;
use IriValue = object::IriValue; export IriValue;
use StringValue = object::StringValue; export StringValue;
use TypedValue = object::TypedValue; export TypedValue;
use UnboundValue = object::UnboundValue; export UnboundValue;

use create_store = store::create_store; export create_store;
use Entry = store::Entry; export Entry;
use ExtensionFn = store::ExtensionFn; export ExtensionFn;
use get_blank_name = store::get_blank_name; export get_blank_name;
use Predicate = store::Predicate; export Predicate;
use make_triple_blank = store::make_triple_blank; export make_triple_blank;
use make_triple_str = store::make_triple_str; export make_triple_str;
use make_triple_uri = store::make_triple_uri; export make_triple_uri;
use Namespace = store::Namespace; export Namespace;
use Store = store::Store; export Store;
use Subject = store::Subject; export Subject;
use Triple = store::Triple; export Triple;
use store_methods = store::store_methods; export store_methods;
//import base_iter = store::base_iter; export base_iter;	// use `import rrdf::store::base_iter` (doing the export here causes "multiple applicable methods in scope" errors)
use to_str = store::to_str; export to_str;

use compile = sparql::compile; export compile;
use Selector = sparql::Selector; export Selector;

use Solution = solution::Solution; export Solution;
use SolutionRow = solution::SolutionRow; export SolutionRow;
use solution_row_methods = solution::solution_row_methods; export solution_row_methods;
use solution_methods = solution::solution_methods; export solution_methods;


