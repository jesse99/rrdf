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
pub fn get_bindings(names: &[~str], algebra: &Algebra) -> ~[~str]
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
	add_algebra_bindings(&mut bindings, algebra);
	
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
		// TODO: 
		// Could speed this up by creating indexes for group2 where the keys are binding name/value and the values are row indexes.
		// Union would then be the intersection of matching row indexes. Another option would be to use tasks (probably would only
		// want to do this for big stores).
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
	
	return Solution {namespaces: copy store.namespaces, bindings: copy group1.bindings, num_selected: group1.num_selected, rows: result};
}

// Attempts to match a pattern to an IRI or blank subject.
priv fn match_subject(bindings: &[~str], actual: &str, pattern: &Pattern, row: &mut SolutionRow) -> bool
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = bindings.position_elem(name);
			assert row[i.get()].is_unbound();					// subject is always the first thing matched so this should always be true
			
			row[i.get()] = if actual.starts_with("_:")
				{
					@BlankValue(actual.to_owned())
				}
				else
				{
					@IriValue(actual.to_owned())
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
priv fn match_predicate(bindings: &[~str], actual: &str, pattern: &Pattern, row: &mut SolutionRow) -> result::Result<bool, ~str>
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = bindings.position_elem(name);
			match row[i.get()]
			{
				@UnboundValue =>
				{
					row[i.get()] = @IriValue(actual.to_owned());
					result::Ok(true)
				}
				_ =>
				{
					result::Err(fmt!("Binding ?%s was set more than once.", *name))
				}
			}
		}
		Constant(@IriValue(ref value)) =>
		{
			//debug!("Actual predicate %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			result::Ok(actual == *value)
		}
		_ =>
		{
			result::Ok(false)
		}
	}
}

// Attempts to match a pattern to an arbitrary object.
priv fn match_object(solution: &Solution, actual: @Object, pattern: &Pattern, row: &mut SolutionRow) -> result::Result<bool, ~str>
{
	match *pattern
	{
		Variable(ref name) =>
		{
			let i = solution.bindings.position_elem(name);
			match row[i.get()]
			{
				@UnboundValue =>
				{
					row[i.get()] = actual;
					result::Ok(true)
				}
				_ =>
				{
					result::Err(fmt!("Binding ?%s was set more than once.", *name))
				}
			}
		}
		Constant(ref expected) =>
		{
			//debug!("Actual object %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], expected.to_str()];
			result::Ok(equal_objects(actual, *expected))
		}
	}
}

// Iterates over all the statements where the subject matches spattern and calls callback for each one.
priv fn iterate_matches(store: &Store, bindings: &[~str], spattern: &Pattern, callback: fn (SolutionRow, &Entry) -> bool)
{
	fn invoke(bindings: &[~str], subject: &str, pattern: &Pattern, entries: @DVec<Entry>, callback: fn (SolutionRow, &Entry) -> bool) -> bool
	{
		for entries.each() |entry|
		{
			let mut row = vec::from_elem(bindings.len(), @UnboundValue);
			if match_subject(bindings, subject, pattern, &mut row)
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
				let entries = option::get(candidate);
				if !invoke(bindings, *subject, spattern, entries, callback)
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
				if !invoke(bindings, *subject, spattern, entries, callback)
				{
					return;
				}
			}
		}
	}
}

// Returns all the subjects that match the TriplePattern.
pub fn eval_basic(store: &Store,  bindings: ~[~str], num_selected: uint, matcher: &TriplePattern) -> result::Result<Solution, ~str>
{
	let mut solution = Solution {namespaces: copy store.namespaces, bindings: copy bindings, num_selected: num_selected, rows: ~[]};
	
	for iterate_matches(store, bindings, &matcher.subject) |r, entry|
	{
		let mut row = move r;		// need the move to shut the borrow checker up
		let result = match_predicate(bindings, entry.predicate, &matcher.predicate, &mut row);
		if result.is_ok() && result.get()
		{
			let result = match_object(&solution, entry.object, &matcher.object, &mut row);
			if result.is_ok() && result.get()
			{
				info!("basic %s matched %s", triple_pattern_to_str(store, matcher), solution_row_to_str(store, &solution, &row));
				solution.rows.push(row);
			}
			else if result.is_err()
			{
				return result::Err(result.get_err());
			}
		}
		else if result.is_err()
		{
			return result::Err(result.get_err());
		}
	}
	
	result::Ok(solution)
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
				if solution.rows[i][j.get()].is_unbound()
				{
					solution.rows[i][j.get()] = value;
				}
				else
				{
					return option::Some(fmt!("Binding ?%s was set more than once.", name));
				}
			}
		}
	}
	
	option::None
}

// Evaluate expr for each row in the solution. If expr returns false the row is removed.
// May return an error message.
priv fn filter_solution(context: &QueryContext, solution: &mut Solution, expr: &Expr) -> option::Option<~str>
{
	let mut i = 0;
	while i < solution.rows.len()
	{
		let value = eval_expr(context, &*solution, &solution.rows[i], expr);
		match get_ebv(value)
		{
			result::Ok(true) =>
			{
				i += 1;
			}
			result::Ok(false) =>
			{
				debug!("FILTER rejected %?", solution.rows[i]);
				solution.rows.swap_remove(i);
			}
			result::Err(copy err) =>
			{
				return option::Some(err);
			}
		}
	}
	
	option::None
}

// Evaluates an optional term against the store. Returns either the solution rows that matched or an empty solution.
priv fn eval_optional(store: &Store, context: &QueryContext, bindings: ~[~str], num_selected: uint, term: &Algebra) -> Solution
{
	match eval_algebra(store, &QueryContext {algebra: copy *term, ..*context}, copy bindings, num_selected)
	{
		result::Ok(move solution) =>
		{
			solution
		}
		result::Err(_) =>
		{
			Solution {namespaces: copy store.namespaces, bindings: copy bindings, num_selected: num_selected, rows: ~[]}
		}
	}
}

// Evaluates the terms against either the store or the current version of the solution. Terms that return new
// solutions join their solution to the current solution. Returns either a solution or an error message.
priv fn eval_group(store: &Store, context: &QueryContext, bindings: ~[~str], num_selected: uint, terms: &[@Algebra]) -> result::Result<Solution, ~str>
{
	let mut result = Solution {namespaces: copy store.namespaces, bindings: copy bindings, num_selected: num_selected, rows: ~[]};
	
	let i = 0;
	while i < terms.len()			// we don't use vec::eachi to work around the borrow checker
	{
		let term = &terms[i];
		info!(" ");
		match term
		{
			&@Filter(ref expr) =>
			{
				match filter_solution(context, &mut result, expr)
				{
					option::None => info!("term%? %s matched %s", i, algebra_to_str(store, *term), solution_to_str(store, &result)),
					option::Some(copy mesg) => return result::Err(mesg),
				}
			}
			&@Bind(ref expr, ref name) =>
			{
				match bind_solution(context, &mut result, expr, copy *name)
				{
					option::None => info!("term%? %s matched %s", i, algebra_to_str(store, *term), solution_to_str(store, &result)),
					option::Some(copy mesg) => return result::Err(mesg),
				}
			}
			_ =>
			{
				match eval_algebra(store, &QueryContext {algebra: copy **term, ..*context}, copy bindings, num_selected)
				{
					result::Ok(move solution) =>
					{
						match **term
						{
							Optional(_t) =>
							{
								if result.rows.is_not_empty()
								{
									result = join_solutions(store, &result, &solution, true);
									info!("term%? %s matched %s", i, algebra_to_str(store, *term), solution_to_str(store, &result));
								}
							}
							_ =>
							{
								if solution.rows.is_empty()
								{
									info!("term%? %s matched nothing", i, algebra_to_str(store, *term));
									return result::Ok(Solution {rows: ~[], ..result});
								}
								else if result.rows.is_not_empty()
								{
									result = join_solutions(store, &result, &solution, false);
									info!("term%? %s matched %s", i, algebra_to_str(store, *term), solution_to_str(store, &result));
								}
								else if i == 0		// the very first pattern in the group has nothing to join with
								{
									result = solution;
									info!("term%? %s matched %s", i, algebra_to_str(store, *term), solution_to_str(store, &result));
								}
							}
						}
					}
					result::Err(copy mesg) =>
					{
						return result::Err(mesg);
					}
				}
			}
		}
	}
	
	return result::Ok(result);
}

// Evaluates the terms against either the store or the current version of the solution. Terms that return new
// solutions join their solution to the current solution. Returns either the solution or an error message.
priv fn eval_algebra(store: &Store, context: &QueryContext, bindings: ~[~str], num_selected: uint) -> result::Result<Solution, ~str>
{
	match context.algebra
	{
		Basic(ref pattern) =>
		{
			eval_basic(store, bindings, num_selected, pattern)
		}
		Group(ref terms) =>
		{
			eval_group(store, context, bindings, num_selected, *terms)
		}
		Optional(term) =>
		{
			result::Ok(eval_optional(store, context, bindings, num_selected, term))
		}
		Bind(*) =>
		{
			result::Err(~"BIND should appear in a pattern group.")
		}
		Filter(*) =>
		{
			// Not sure what's supposed to happen here. According to GroupGraphPatternSub a
			// group can contain just a FILTER (should be a no-op?) or a filter and then a triple
			// pattern (filter position doesn't matter?).
			result::Err(~"FILTER should appear last in a pattern group.")
		}
	}
}

// Returns either the solution sorted using exprs or an error message.
priv fn order_by(context: &QueryContext, solution: Solution, ordering: &[Expr]) -> result::Result<Solution, ~str>
{
	pure fn compare_rows(err_mesg: @mut ~str, ordering: &[Expr], context: &QueryContext, solution: &Solution, row1: &SolutionRow, row2: &SolutionRow) -> bool
	{
		pure fn compare_order_values(lhs: &(bool, @Object), rhs: &(bool, @Object)) -> result::Result<int, ~str>
		{
			assert lhs.first() == rhs.first();
			
			match *lhs
			{
				(true, ref x) =>
				{
					compare_values(~"<", *x, rhs.second())		// ascending
				}
				(false, ref x) =>
				{
					compare_values(~"<", rhs.second(), *x)		// descending
				}
			}
		}
		
		pure fn eval_order_expr(context: &QueryContext, solution: &Solution, row: &SolutionRow, expr: &Expr) -> (bool, @Object)
		{
			match *expr
			{
				CallExpr(~"!desc", ref e) => (false, eval_expr(context, solution, row, e[0])),
				CallExpr(~"!asc", ref e) => (true, eval_expr(context, solution, row, e[0])),
				_ => (true, eval_expr(context, solution, row, expr)),
			}
		}
		
		let order1 = vec::map(ordering, |o| {eval_order_expr(context, solution, row1, o)});
		let order2 = vec::map(ordering, |o| {eval_order_expr(context, solution, row2, o)});
		let order = vec::map2(order1, order2, |x, y| {compare_order_values(x, y)});
		let order = do vec::foldl(result::Ok(0), order) |x, y|
		{
			match x
			{
				result::Ok(0)	=>	copy *y,
				_			 	=> x,
			}
		};
		match order
		{
			result::Ok(x) =>
			{
				x < 0
			}
			result::Err(copy err) =>
			{
				if str::is_empty(*err_mesg)
				{
					unsafe {*err_mesg = err;}
				}
				false
			}
		}
	}
	
	// TODO: once quick_sort is fixed to use inherited mutability we should be able to switch to that
	let err_mesg = @mut ~"";
	let rows = std::sort::merge_sort(solution.rows, |x, y| {compare_rows(err_mesg, ordering, context, &solution, x, y)});
	if str::is_empty(*err_mesg)
	{
		result::Ok(Solution {rows: rows, ..solution})
	}
	else
	{
		result::Err(copy *err_mesg)
	}
}

priv fn make_distinct(solution: Solution) -> result::Result<Solution, ~str>
{
	pure fn equal_rows(solution: &Solution, row1: &SolutionRow, row2: &SolutionRow) -> bool
	{
		for uint::range(0, solution.num_selected) |i|
		{
			if row1[i] != row2[i]
			{
				return false;
			}
		}
		true
	}
	
	// TODO: Could skip this, but only if the user uses ORDER BY for every variable in the result.
	// TODO: once quick_sort is fixed to use inherited mutability we should be able to switch to that
	let rows = std::sort::merge_sort(solution.rows, |x, y| {x <= y});
	
	let mut result = ~[];
	vec::reserve(&mut result, rows.len());
	
	let mut i = 0;
	while i < rows.len()
	{
		let row = copy rows[i];
		vec::push(&mut result, copy row);
		
		i = i + 1;
		while i < rows.len() && equal_rows(&solution, &row, &rows[i])
		{
			i += 1;
		}
	}
	
	return result::Ok(Solution {rows: result, ..solution});
}

// Creates a closure which will evaulate the terms in context against a store passed into the closure.
// names are from the SELECT clause
pub fn eval(names: &[~str], context: &QueryContext) -> Selector
{
	let context = copy *context;
	let (bindings, num_selected) = if names.len() == 1 && names[0] == ~"*"
		{
			let tmp = get_bindings(~[], &context.algebra);
			let len = tmp.len();
			(tmp, len)
		}
		else
		{
			(get_bindings(names, &context.algebra), names.len())
		};
	|store: &Store, move bindings|
	{
		info!("algebra: %s", algebra_to_str(store, &context.algebra));
		let context = QueryContext {namespaces: copy store.namespaces, extensions: store.extensions, ..copy context};
		do eval_algebra(store, &context, copy bindings, num_selected).chain() |solution|
		{
			// Optionally remove duplicates.
			do result::chain(if context.distinct {make_distinct(solution)} else {result::Ok(move solution)})
			|solution|
			{
				// Optionally sort the solution.
				do result::chain(if vec::is_not_empty(context.order_by) {order_by(&context, solution, context.order_by)} else {result::Ok(move solution)})
				|solution|
				{
					match context.limit
					{
						// Optionally limit the solution.
						option::Some(limit) if limit < vec::len(solution.rows) =>
						{
							result::Ok(Solution {rows: vec::slice(solution.rows, 0, limit), ..solution})
						}
						_ =>
						{
							result::Ok(move solution)
						}
					}
				}
			}
		}
	}
}
