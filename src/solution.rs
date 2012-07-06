#[doc = "The result of a SPARQL query."];

#[doc = "Result of matching a triple with a SPARQL query."]
type solution_row = [(str, object)];

#[doc = "Result of a SPARQL query.

Note that the solution_methods impl provides a number of convenience methods
to simplify result retrieval."]
type solution = [solution_row];

impl solution_row_methods for solution_row
{
	pure fn get(name: str) -> object
	{
		alt vec::find(self, {|e| tuple::first(e) == name})
		{
			option::some(result)
			{
				tuple::second(result)
			}
			option::none
			{
				fail("Couldn't find " + name)
			}
		}
	}
	
	// Named search so we don't wind up conflicting with the find vec extension.
	pure fn search(name: str) -> option<object>
	{
		alt vec::find(self, {|e| tuple::first(e) == name})
		{
			option::some(result)
			{
				option::some(tuple::second(result))
			}
			option::none
			{
				option::none
			}
		}
	}
}

impl solution_methods for solution
{
	pure fn get(row: uint, name: str) -> object
	{
		self[row].get(name)
	}
	
	pure fn search(row: uint, name: str) -> option<object>
	{
		self[row].search(name)
	}
}
