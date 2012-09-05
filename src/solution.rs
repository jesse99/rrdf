//! The result of a SPARQL query.
use Option = option::Option;
use object::*;

// Note that solutions must be sendable types so that queries can be off-loaded
// onto tasks.

/// Result of matching a triple with a SPARQL query.
///
/// Order of entries in each row will match the order in the SELECT clause.
type solution_row = ~[(~str, object)];

/// Result of a SPARQL query.
/// 
/// Note that the solution_methods impl provides a number of convenience methods
/// to simplify result retrieval.
type solution = ~[solution_row];

trait solution_trait
{
	pure fn get(row: uint, name: ~str) -> object;
	pure fn search(row: uint, name: ~str) -> Option<object>;
}

trait solution_row_trait
{
	pure fn get(name: ~str) -> object;
	pure fn contains(name: ~str) -> bool;
	pure fn search(name: ~str) -> Option<object>;
}

impl  solution : solution_trait 
{
	pure fn get(row: uint, name: ~str) -> object
	{
		self[row].get(name)
	}
	
	pure fn search(row: uint, name: ~str) -> Option<object>
	{
		self[row].search(name)
	}
}

impl  solution_row : solution_row_trait 
{
	pure fn get(name: ~str) -> object
	{
		match vec::find(self, |e| {e.first() == name})
		{
			option::Some(result) =>
			{
				result.second()
			}
			option::None =>
			{
				fail(~"Couldn't find " + name)
			}
		}
	}
	
	pure fn contains(name: ~str) -> bool
	{
		vec::find(self, |e| {e.first() == name}).is_some()
	}
	
	// Named search so we don't wind up conflicting with the find vec extension.
	pure fn search(name: ~str) -> Option<object>
	{
		match vec::find(self, |e| {e.first() == name})
		{
			option::Some(result) =>
			{
				option::Some(result.second())
			}
			option::None =>
			{
				option::None
			}
		}
	}
}
