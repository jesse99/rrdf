//! The result of a SPARQL query.

/// Name of a namespace plus the IRI it expands to.
///
/// This is a sendable type.
pub struct Namespace {prefix: ~str, path: ~str}

/// Result of matching a triple with a SPARQL query.
pub type SolutionRow = ~[@Object];

/// Result of a SPARQL query.
/// 
/// Bindings contains the names of variables for each row. The order of the names
/// will match the order in the SELECT clause. To allow for faster query execution
/// there may be more bindings than those listed in the SELECT clause. Note that 
/// this is a sendable type.
pub struct Solution
{
	pub namespaces: ~[Namespace],
	pub bindings: ~[~str],
	pub num_selected: uint,
	pub rows: ~[SolutionRow],
}

pub trait SolutionMethods
{
	pure fn get(row: uint, name: ~str) -> @Object;
	pure fn find(row: uint, name: ~str) -> Option<@Object>;
	
	/// In general an ORDER BY clause should be used to sort solutions.
	/// However it can be convenient to manually sort them for things
	/// like unit tests.
	pure fn sort() -> Solution;
}

pub impl  &Solution : SolutionMethods
{
	pure fn get(row: uint, name: ~str) -> @Object
	{
		match self.bindings.position_elem(&name)
		{
			option::Some(i) =>
			{
				self.rows[row][i]
			}
			option::None =>
			{
				fail fmt!("%s isn't one of the bindings (%s)", name, str::connect(self.bindings, ", "))
			}
		}
	}
	
	pure fn find(row: uint, name: ~str) -> Option<@Object>
	{
		match self.bindings.position_elem(&name)
		{
			option::Some(i) =>
			{
				option::Some(self.rows[row][i])
			}
			option::None =>
			{
				option::None
			}
		}
	}
	
	pure fn sort() -> Solution
	{
		pure fn solution_row_le(x: &SolutionRow, y: &SolutionRow) -> bool
		{
			unsafe
			{
				for x.eachi |i, xx|
				{
					let r = operators::compare_values(~"sort", *xx, y[i]);
					if r == result::Ok(-1)
					{
						return true;
					}
					else if r == result::Ok(1)
					{
						return false;
					}
				}
				true
			}
		}
		
		unsafe
		{
			Solution {namespaces: copy self.namespaces, bindings: copy self.bindings, num_selected: self.num_selected, rows: std::sort::merge_sort(solution_row_le, self.rows)}
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
		
		for self.rows.eachi |i, row|
		{
			let row = row.slice(0, self.num_selected);
			let entries = do row.mapi |i, r| {fmt!("%s: %s", self.bindings[i], r.to_friendly_str(self.namespaces))};
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
