//! The result of a SPARQL query.
use Option = option::Option;
use object::*;

// Note that solutions must be sendable types so that queries can be off-loaded
// onto tasks.

/// Result of matching a triple with a SPARQL query.
///
/// Order of entries in each row will match the order in the SELECT clause.
type SolutionRow = ~[(~str, Object)];

/// Result of a SPARQL query.
/// 
/// Note that the solution_methods impl provides a number of convenience methods
/// to simplify result retrieval.
type Solution = ~[SolutionRow];

trait SolutionTrait
{
	pure fn get(row: uint, name: ~str) -> Object;
	pure fn search(row: uint, name: ~str) -> Option<Object>;
	
	/// In general an ORDER BY clause should be used to sort solutions.
	/// However it can be convenient to manually sort them for things
	/// like unit tests.
	pure fn sort() -> Solution;
}

trait SolutionRowTrait
{
	pure fn get(name: ~str) -> Object;
	pure fn contains(name: ~str) -> bool;
	pure fn search(name: ~str) -> Option<Object>;
}

impl  Solution : SolutionTrait 
{
	pure fn get(row: uint, name: ~str) -> Object
	{
		self[row].get(name)
	}
	
	pure fn search(row: uint, name: ~str) -> Option<Object>
	{
		self[row].search(name)
	}
	
	pure fn sort() -> Solution
	{
		pure fn solution_row_le(x: &SolutionRow, y: &SolutionRow) -> bool
		{
			unchecked
			{
				if x.len() < y.len()
				{
					true
				}
				else if x.len() > y.len()
				{
					false
				}
				else
				{
					for x.eachi
					|i, xx|
					{
						if xx.first() < y[i].first()
						{
							return true;
						}
						else if xx.first() > y[i].first()
						{
							return false;
						}
						
						let r = operators::compare_values(~"sort", xx.second(), y[i].second());
						if r == result::Ok(-1)
						{
							return true;
						}
						else if r == result::Ok(1)
						{
							return false;
						}
					}
					true		// everything was equal
				}
			}
		}
		
		unchecked
		{
			std::sort::merge_sort(solution_row_le, self)
		}
	}
}

impl  SolutionRow : SolutionRowTrait 
{
	pure fn get(name: ~str) -> Object
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
	pure fn search(name: ~str) -> Option<Object>
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
