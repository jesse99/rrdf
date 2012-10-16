//! The result of a SPARQL query.
use Option = option::Option;
use object::*;

/// Name of a namespace plus the IRI it expands to.
///
/// This is a sendable type.
pub struct Namespace {prefix: ~str, path: ~str}

/// Result of matching a triple with a SPARQL query.
///
/// Order of entries in each row will match the order in the SELECT clause.
pub type SolutionRow = ~[(~str, Object)];

/// Result of a SPARQL query.
/// 
/// Note that this is a sendable type.
pub struct Solution
{
	pub namespaces: ~[Namespace],
	pub rows: ~[SolutionRow],
}

pub trait SolutionMethods
{
	pure fn get(row: uint, name: &str) -> Object;
	pure fn search(row: uint, name: &str) -> Option<Object>;
	
	/// In general an ORDER BY clause should be used to sort solutions.
	/// However it can be convenient to manually sort them for things
	/// like unit tests.
	pure fn sort() -> Solution;
}

pub trait SolutionRowMethods
{
	pure fn get(name: &str) -> Object;
	pure fn contains(name: &str) -> bool;
	pure fn search(name: &str) -> Option<Object>;
}

pub impl  &Solution : SolutionMethods
{
	pure fn get(row: uint, name: &str) -> Object
	{
		self.rows[row].get(name)
	}
	
	pure fn search(row: uint, name: &str) -> Option<Object>
	{
		self.rows[row].search(name)
	}
	
	pure fn sort() -> Solution
	{
		pure fn solution_row_le(x: &SolutionRow, y: &SolutionRow) -> bool
		{
			unsafe
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
						
						let r = operators::compare_values(~"sort", &xx.second(), &y[i].second());
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
		
		unsafe
		{
			Solution {namespaces: copy self.namespaces, rows: std::sort::merge_sort(solution_row_le, self.rows)}
		}
	}
}

// TODO: This should be in the impl above, but is not because of
// https://github.com/mozilla/rust/issues/3410
pub impl  &Solution : ToStr
{
	pure fn to_str() -> ~str
	{
		let mut result = ~"";		// TODO: need to replace this with some sort of StringBuilder
		
		for self.rows.eachi
		|i, row|
		{
			let entries = do row.map |r| {fmt!("%s: %s", r.first(), r.second().to_friendly_str(self.namespaces))};
			result += fmt!("%? %s\n", i, str::connect(entries, ", "));
		}
		
		result
	}
}

pub impl Namespace : cmp::Eq
{
	pure fn eq(other: &Namespace) -> bool
	{
		self.prefix == other.prefix && self.path == other.path
	}
	
	pure fn ne(other: &Namespace) -> bool
	{
		!self.eq(other)
	}
}

pub impl Solution : cmp::Eq
{
	pure fn eq(other: &Solution) -> bool
	{
		self.namespaces == other.namespaces && self.rows == other.rows
	}
	
	pure fn ne(other: &Solution) -> bool
	{
		!self.eq(other)
	}
}

pub impl  &[(~str, Object)] : SolutionRowMethods 
{
	pure fn get(name: &str) -> Object
	{
		match vec::find(self, |e| {str::eq_slice(e.first(), name)})
		{
			option::Some(ref result) =>
			{
				result.second()
			}
			option::None =>
			{
				fail(~"Couldn't find " + name)
			}
		}
	}
	
	pure fn contains(name: &str) -> bool
	{
		vec::find(self, |e| {str::eq_slice(e.first(), name)}).is_some()
	}
	
	// Named search so we don't wind up conflicting with the find vec extension.
	pure fn search(name: &str) -> Option<Object>
	{
		match vec::find(self, |e| {str::eq_slice(e.first(), name)})
		{
			option::Some(ref result) =>
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
