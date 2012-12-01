//! Used when evaluating a SPARQL query. Clients will not ordinarily use this.
// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
use expression::*;
use operators::*;

pub enum Pattern
{
	Variable(~str),
	Constant(@Object)
}

pub struct TriplePattern {subject: Pattern, predicate: Pattern, object: Pattern}

pub enum Algebra
{
	Basic(TriplePattern),
	Group(~[@Algebra]),
	Optional(@Algebra),
	Bind(expression::Expr, ~str),
	Filter(expression::Expr)
}

pub struct QueryContext
{
	pub namespaces: ~[Namespace],
	pub extensions: HashMap<@~str, ExtensionFn>,
	pub algebra: Algebra,
	pub order_by: ~[expression::Expr],
	pub distinct: bool,
	pub limit: Option<uint>,
	pub rng: rand::Rng,		// for RAND
	pub timestamp: Tm,		// for NOW
}

// TODO: All of these functions except eval should be private. But then we'd have to move the unit tests
// here which kind of blows. See https://github.com/mozilla/rust/issues/3505

/// The function returned by compile and invoked to execute a SPARQL query. 
/// 
/// Returns a solution or a 'runtime' error.
pub type Selector = fn@ (s: &Store) -> result::Result<Solution, ~str>;

// Returns the names (from the SELECT clause) followed by bound variable
// names not in names.
pub fn get_bindings(names: &[~str], algebra: Algebra) -> ~[~str]
{
	fn add_pattern_binding(bindings: &mut ~[~str], pattern: Pattern)
	{
		match pattern
		{
			Variable(ref v) =>
			{
				if !bindings.contains(v)
				{
					bindings.push(copy *v);
				}
			}
			Constant(_) =>
			{
			}
		}
	}
	
	fn add_algebra_bindings(bindings: &mut ~[~str], algebra: &Algebra)
	{
		match *algebra
		{
			Basic(ref v) =>
			{
				add_pattern_binding(bindings, copy v.subject);
				add_pattern_binding(bindings, copy v.predicate);
				add_pattern_binding(bindings, copy v.object);
			}
			Group(ref v) =>
			{
				for v.each |a| {add_algebra_bindings(bindings, *a)};
			}
			Optional(v) =>
			{
				add_algebra_bindings(bindings, v);
			}
			Bind(_, ref v) =>
			{
				if !bindings.contains(v) 
				{
					bindings.push(copy *v);
				}
			}
			Filter(_) =>
			{
			}
		}
	}
	
	let mut bindings = ~[];
	bindings.push_all(names);
	add_algebra_bindings(&mut bindings, &algebra);
	
	return bindings;
}

pub fn pattern_to_str(store: &Store, pattern: &Pattern) -> ~str
{
	match *pattern
	{
		Variable(ref v) =>
		{
			fmt!("?%s", *v)
		}
		Constant(ref c) =>
		{
			c.to_friendly_str(store.namespaces)
		}
	}
}

pub fn triple_pattern_to_str(store: &Store, pattern: &TriplePattern) -> ~str
{
	fmt!("{subject: %s, predicate: %s, object: %s}", pattern_to_str(store, &pattern.subject), pattern_to_str(store, &pattern.predicate), pattern_to_str(store, &pattern.object))
}

pub fn algebra_to_str(store: &Store, algebra: &Algebra) -> ~str
{
	match *algebra
	{
		Basic(ref p) =>
		{
			triple_pattern_to_str(store, p)
		}
		Group(ref args) =>
		{
			fmt!("[%s]", str::connect(do args.map |a| {algebra_to_str(store, *a)}, ~", "))
		}
		Optional(a) =>
		{
			~"optional " + algebra_to_str(store, a)
		}
		Bind(ref e, ref n) =>
		{
			fmt!("%s = %s", *n, expr_to_str(store, e))
		}
		Filter(ref e) =>
		{
			~"filter " + expr_to_str(store, e)
		}
	}
}

pub fn solution_row_to_str(store: &Store, solution: &Solution, row: &SolutionRow) -> ~str
{
	let mut entries = ~[];
	for row.eachi |i, entry|
	{
		let name = copy solution.bindings[i];
		let value = entry.to_friendly_str(store.namespaces);
		let prefix = if i == solution.num_selected {"["} else {""};
		let suffix = if i == solution.bindings.len() - 1 && solution.bindings.len() > solution.num_selected {"]"} else {""};
		vec::push(&mut entries, fmt!("%s%s: %s%s", prefix, name, value, suffix));
	};
	str::connect(entries, ~", ")
}

pub fn solution_to_str(store: &Store, solution: &Solution) -> ~str
{
	let mut result = ~"";
	
	for vec::eachi(solution.rows) |i, row|
	{
		result += fmt!("%?: %s   ", i, solution_row_to_str(store, solution, row));
	};
	
	if result.is_empty()
	{
		result = ~"nothing";
	}
	
	return result;
}

priv fn equal_objects(actual: &Object, expected: &Object) -> bool
{
	match op_equals(actual, expected)	// should get BoolValue or ErrorValue
	{
		BoolValue(value) =>
		{
			value
		}
		_ =>
		{
			false
		}
	}
}

// This returns the union of each row in group1 with compatible rows from group2. Where rows are
// compatible if the values that they bind are equal. For normal triple pattern groups optional_join
// will be false which means the returned rows are only the ones that were able to be unioned. For 
// OPTIONAL pattern groups optional_join will be true when in which case rows from group1 are 
// returned even when they were not unioned.
//
// Here's an example:
// Group1							Group2
// Name		Occupation		Name			Hobbies
// John Smith	Dentist				Jane Doe		Yak Shaving		note that Jane Doe is never returned (because it is incompatible with every group1 row)
// Bill Connor	Carpenter			John Smith		Gardening
// Jill Jackson	Sys Admin			Bill Connor	Model trains
//
// With optional_join off result will be:
// Name		Occupation		Hobbies
// John Smith	Dentist				Gardening
// Bill Connor	Carpenter			Model trains
//
// With optional_join on result will be:
// Name		Occupation		Hobbies
// John Smith	Dentist				Gardening
// Bill Connor	Carpenter			Model trains
// Jill Jackson	Sys Admin			<not bound>
pub fn join_solutions(store: &Store, group1: &Solution, group2: &Solution, optional_join: bool) -> Solution
{
	fn compatible_row(bindings: &[~str], lhs: &SolutionRow, rhs: &SolutionRow) -> bool
	{
		for uint::range(0, bindings.len()) |i|
		{
			if !lhs[i].is_unbound() && !rhs[i].is_unbound()
			{
				if !equal_objects(lhs[i], rhs[i])
				{
					return false;
				}
			}
		}
		return true;
	}
	
	fn union_rows(bindings: &[~str], lhs: &SolutionRow, rhs: &SolutionRow) -> SolutionRow
	{
		let mut result = copy(*lhs);
		
		for uint::range(0, bindings.len()) |i|
		{
			if lhs[i].is_unbound() && !rhs[i].is_unbound()
			{
				result[i] = rhs[i];
			}
		}
		
		return result;
	}
	
	assert group1.bindings.len() == group2.bindings.len();

	let mut result = ~[];
	info!("joining:");
	info!("   group1 = %s", solution_to_str(store, group1));
	info!("   group2 = %s", solution_to_str(store, group2));
	if vec::is_not_empty(group1.rows) && (vec::is_not_empty(group2.rows) || optional_join)
	{
		for vec::each(group1.rows) |lhs|
		{
			let count = vec::len(result);
			for vec::each(group2.rows) |rhs|
			{
				if compatible_row(group1.bindings, lhs, rhs)
				{
					let unioned = union_rows(group1.bindings, lhs, rhs);
					info!("   added: %s", solution_row_to_str(store, group1, &unioned));
					vec::push(&mut result, unioned);
				}
				else
				{
					debug!("   not compatible: %s and %s", solution_row_to_str(store, group1, lhs), solution_row_to_str(store, group1, rhs));
				}
			}
			if vec::len(result) == count && optional_join
			{
				// With OPTIONAL we need to add the lhs row even if we failed to find
				// any compatible rhs rows.
				info!("   optional: %s", solution_row_to_str(store, group1, lhs));
				vec::push(&mut result, copy *lhs);
			}
		}
	}
	
	if result.is_empty()
	{
		info!("   empty result");
	}
	
	return Solution {namespaces: copy store.namespaces, bindings: group1.bindings, num_selected: group1.num_selected, rows: result};
}

// Attempts to match a pattern to an IRI or blank subject.
priv fn match_subject(solution: &Solution, actual: &str, pattern: &Pattern, row: &mut SolutionRow) -> bool
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = solution.bindings.position_elem(name);
			row[i.get()] = if actual.starts_with("_:")
				{
					@BlankValue(actual.to_unique())
				}
				else
				{
					@IriValue(actual.to_unique())
				};
			true
		}
		Constant(@IriValue(ref value)) =>
		{
			//debug!("Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			actual == *value
		}
		Constant(@BlankValue(ref value)) =>
		{
			//debug!("Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			actual == *value
		}
		_ =>
		{
			false
		}
	}
}

// Attempts to match a pattern to an IRI predicate.
priv fn match_predicate(solution: &Solution, actual: &str, pattern: &Pattern, row: &mut SolutionRow) -> bool
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = solution.bindings.position_elem(name);
			row[i.get()] = @IriValue(actual.to_unique());
			true
		}
		Constant(@IriValue(ref value)) =>
		{
			//debug!("Actual predicate %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			actual == *value
		}
		_ =>
		{
			false
		}
	}
}

// Attempts to match a pattern to an arbitrary object.
priv fn match_object(solution: &Solution, actual: @Object, pattern: &Pattern, row: &mut SolutionRow) -> bool
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = solution.bindings.position_elem(name);
			row[i.get()] = actual;
			true
		}
		Constant(ref expected) =>
		{
			//debug!("Actual object %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], expected.to_str()];
			equal_objects(actual, *expected)
		}
	}
}

// Iterates over all the statements where the subject matches spattern and calls callback for each one.
priv fn iterate_matches(store: &Store, solution: &Solution, spattern: &Pattern, callback: fn (SolutionRow, &Entry) -> bool)
{
	fn invoke(solution: &Solution, subject: &str, pattern: &Pattern, entries: @DVec<Entry>, callback: fn (SolutionRow, &Entry) -> bool) -> bool
	{
		for entries.each() |entry|
		{
			let mut row = vec::from_elem(solution.bindings.len(), @UnboundValue);
			if match_subject(solution, subject, pattern, &mut row)
			{
				if !callback(move row, entry)
				{
					return false;
				}
			}
		}
		true
	}
	
	match *spattern
	{
		Constant(@IriValue(ref subject)) | Constant(@BlankValue(ref subject)) =>
		{
			// Optimization for a common case where we are attempting to match a specific subject.
			let candidate = store.subjects.find(@copy *subject);
			if option::is_some(&candidate)
			{
				info!("--- matched subject %?", subject);
				let entries = option::get(&candidate);
				if !invoke(solution, *subject, spattern, entries, callback)
				{
					return;
				}
			}
		}
		_ =>
		{
			for store.subjects.each() |subject, entries|
			{
				debug!("--- trying subject %?", subject);
				if !invoke(solution, *subject, spattern, entries, callback)
				{
					return;
				}
			}
		}
	}
}

// Returns all the subjects that match the TriplePattern.
pub fn eval_basic(store: &Store,  bindings: ~[~str], num_selected: uint, matcher: &TriplePattern) -> Solution
{
	let mut solution = Solution {namespaces: copy store.namespaces, bindings: move bindings, num_selected: num_selected, rows: ~[]};
	
	for iterate_matches(store, &solution, &matcher.subject) |r, entry|
	{
		let mut row = move r;		// need the move to shut the borrow checker up
		if match_predicate(&solution, entry.predicate, &matcher.predicate, &mut row)
		{
			if match_object(&solution, entry.object, &matcher.object, &mut row)
			{
				info!("basic %s matched %s", triple_pattern_to_str(store, matcher), solution_row_to_str(store, &solution, &row));
				solution.rows.push(row);
			}
		}
	}
	
	solution
}

// Evaluate expr for each row in the solution and bind the result to name.
// May return an error message.
priv fn bind_solution(context: &QueryContext, solution: &mut Solution, expr: &Expr, name: ~str) -> option::Option<~str>
{
	for uint::range(0, solution.rows.len()) |i|
	{
		let value = eval_expr(context, &*solution, &solution.rows[i], expr);
		match *value
		{
			UnboundValue =>
			{
				return option::Some(~"unbound variable");		// shouldn't hit this case
			}
			InvalidValue(ref literal, ref kind) =>
			{
				return option::Some(fmt!("?%s is not a valid %s", *literal, *kind));
			}
			ErrorValue(copy mesg) =>
			{
				return option::Some(mesg);
			}
			_ =>
			{
				let j = solution.bindings.position_elem(&name);
				solution.rows[i][j.get()] = value
			}
		}
	}
	
	option::None
}
