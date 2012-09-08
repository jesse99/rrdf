//! API for the rrdf library.
use to_str::to_str;
use core::dvec::*;
use std::map::hashmap; 
use std::time::tm;

// TODO: Hopefully we can clean this up a lot when exporting works a bit better.

// store
use Subject = store::Subject; export Subject;
use Predicate = store::Predicate; export Predicate;
use Triple = store::Triple; export Triple;
use Entry = store::Entry; export Entry;
use ExtensionFn = store::ExtensionFn; export ExtensionFn;
use Store = store::Store; export Store;
use get_blank_name = store::get_blank_name; export get_blank_name;
use contract_uri = store::contract_uri; export contract_uri;

// solution
use Namespace = store::Namespace; export Namespace;
use SolutionRow = solution::SolutionRow; export SolutionRow;
use Solution = solution::Solution; export Solution;
use SolutionRowMethods = solution::SolutionRowMethods; export SolutionRowMethods;

// object
use Object = object::Object; export Object;
use BoolValue = object::BoolValue; export BoolValue;
use IntValue = object::IntValue; export IntValue;
use FloatValue = object::FloatValue; export FloatValue;
use DateTimeValue = object::DateTimeValue; export DateTimeValue;
use StringValue = object::StringValue; export StringValue;
use TypedValue = object::TypedValue; export TypedValue;
use IriValue = object::IriValue; export IriValue;
use BlankValue = object::BlankValue; export BlankValue;
use UnboundValue = object::UnboundValue; export UnboundValue;
use InvalidValue = object::InvalidValue; export InvalidValue;
use ErrorValue = object::ErrorValue; export ErrorValue;
use literal_to_object = object::literal_to_object; export literal_to_object;

// sparql
use compile = sparql::compile; export compile;

// query
use Selector = sparql::Selector; export Selector;
