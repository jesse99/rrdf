//! The result of a SPARQL query.

/// Result of matching a triple with a SPARQL query.
type solution_row = ~[(~str, object)];

/// Result of a SPARQL query.
/// 
/// Note that the solution_methods impl provides a number of convenience methods
/// to simplify result retrieval.
type solution = ~[solution_row];

trait solution_row_trait
{
	pure fn get(name: ~str) -> object;
	pure fn search(name: ~str) -> option<object>;
}

impl solution_row_methods of solution_row_trait for solution_row
{
	pure fn get(name: ~str) -> object
	{
		alt vec::find(self, |e| {e.first() == name})
		{
			option::some(result)
			{
				result.second()
			}
			option::none
			{
				fail(~"Couldn't find " + name)
			}
		}
	}
	
	// Named search so we don't wind up conflicting with the find vec extension.
	pure fn search(name: ~str) -> option<object>
	{
		alt vec::find(self, |e| {e.first() == name})
		{
			option::some(result)
			{
				option::some(result.second())
			}
			option::none
			{
				option::none
			}
		}
	}
}

trait solution_trait
{
	pure fn get(row: uint, name: ~str) -> object;
	pure fn search(row: uint, name: ~str) -> option<object>;
}

impl solution_methods of solution_trait for solution
{
	pure fn get(row: uint, name: ~str) -> object
	{
		self[row].get(name)
	}
	
	pure fn search(row: uint, name: ~str) -> option<object>
	{
		self[row].search(name)
	}
}
