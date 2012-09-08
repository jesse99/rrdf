//! Used when evaluating a SPARQL query. Clients will not ordinarily use this.
// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
use std::map::*;
use dvec::*;
use expression::*;
use object::*;
use operators::*;
use solution::*;
use store::*;

export join_solutions, eval, Pattern, Variable, Constant, Algebra, TriplePattern, QueryContext,
	Basic, Group, Optional, Bind, Filter, Selector;

enum Pattern
{
	Variable(~str),
	Constant(Object)
}

type TriplePattern = {subject: Pattern, predicate: Pattern, object: Pattern};

enum Algebra
{
	Basic(TriplePattern),
	Group(~[@Algebra]),
	Optional(@Algebra),
	Bind(expression::Expr, ~str),
	Filter(expression::Expr)
}

type QueryContext =
{
	namespaces: @~[Namespace],
	extensions: @hashmap<@~str, ExtensionFn>,
	algebra: Algebra,
	order_by: ~[expression::Expr],
	distinct: bool,
	limit: Option<uint>,
	rng: rand::Rng,		// for RAND
	timestamp: Tm		// for NOW
};

/// The function returned by compile and invoked to execute a SPARQL query. 
/// 
/// Returns a solution or a 'runtime' error.
type Selector = fn@ (s: &Store) -> result::Result<Solution, ~str>;

type Binding = {name: ~str, value: Object};

type Match = either::Either<Binding, bool>;	// match succeeded if bindings or true

fn pattern_to_str(store: &Store, pattern: Pattern) -> ~str
{
	match pattern
	{
		Variable(v) =>
		{
			fmt!("?%s", v)
		}
		Constant(c) =>
		{
			c.to_friendly_str(store.namespaces)
		}
	}
}

fn triple_pattern_to_str(store: &Store, pattern: TriplePattern) -> ~str
{
	fmt!("{subject: %s, predicate: %s, object: %s}", pattern_to_str(store, pattern.subject), pattern_to_str(store, pattern.predicate), pattern_to_str(store, pattern.object))
}
	
fn algebra_to_str(store: &Store, algebra: &Algebra) -> ~str
{
	match *algebra
	{
		Basic(p) =>
		{
			triple_pattern_to_str(store, p)
		}
		Group(args) =>
		{
			fmt!("[%s]", str::connect(do args.map |a| {algebra_to_str(store, a)}, ~", "))
		}
		Optional(a) =>
		{
			~"optional " + algebra_to_str(store, a)
		}
		Bind(e, n) =>
		{
			fmt!("%s = %s", n, expr_to_str(store, e))
		}
		Filter(e) =>
		{
			~"filter " + expr_to_str(store, e)
		}
	}
}

fn solution_row_to_str(store: &Store, row: SolutionRow) -> ~str
{
	let mut entries = ~[];
	for row.each
	|entry|
	{
		let name = entry.first();
		let value = entry.second().to_friendly_str(store.namespaces);
		vec::push(entries, fmt!("%s: %s", name, value));
	};
	str::connect(entries, ~", ")
}

fn solution_to_str(store: &Store, solution: Solution) -> ~str
{
	let mut result = ~"";
	
	for vec::eachi(solution.rows)
	|i, row|
	{
		result += fmt!("%?: %s   ", i, solution_row_to_str(store, row));
	};
	
	if result.is_empty()
	{
		result = ~"nothing";
	}
	
	return result;
}

// Conceptually treats SolutionRow as a set where each set value consists of both
// the name and the value. Takes the cross product of entries from each pair
// of groups and adds compatible results to the result.
//
// Where a cross product is compatible if, for every identical name, the values
// are also identical.
fn join_solutions(store: &Store, names: ~[~str], group1: Solution, group2: Solution, optional_join: bool) -> Solution
{
	fn compatible_binding(name1: ~str, value1: Object, rhs: SolutionRow) -> bool
	{
		match rhs.search(name1)
		{
			option::Some(value2) =>
			{
				equal_objects(value1, value2)
			}
			option::None() =>
			{
				true
			}
		}
	}
	
	fn compatible_row(row: SolutionRow, rhs: SolutionRow) -> bool
	{
		for row.each()
		|entry|
		{
			if !compatible_binding(entry.first(), entry.second(), rhs)
			{
				return false;
			}
		}
		return true;
	}
	
	fn union_rows(lhs: SolutionRow, rhs: SolutionRow) -> SolutionRow
	{
		let mut result = copy(lhs);
		
		for rhs.each()
		|entry2|
		{
			match lhs.search(entry2.first())
			{
				option::Some(_) =>
				{
					// Binding should be compatible with lhs so nothing to do here.
				}
				option::None() =>
				{
					// This is a binding in rhs but not lhs, so we need to add it to the result.
					vec::push(result, entry2);
				}
			}
		}
		
		return result;
	}
	
	let mut result = ~[];
	info!("joining:");
	info!("   group1 = %s", solution_to_str(store, group1));
	info!("   group2 = %s", solution_to_str(store, group2));
	if vec::is_not_empty(group1.rows) && (vec::is_not_empty(group2.rows) || optional_join)
	{
		for vec::each(group1.rows)
		|lhs|
		{
			let count = vec::len(result);
			for vec::each(group2.rows)
			|rhs|
			{
				if compatible_row(lhs, rhs)
				{
					let unioned = union_rows(lhs, rhs);
					info!("   added: %s", solution_row_to_str(store, unioned));
					vec::push(result, filter_row(names, unioned));
				}
				else
				{
					debug!("   not compatible: %s and %s", solution_row_to_str(store, lhs), solution_row_to_str(store, rhs));
				}
			}
			if vec::len(result) == count && optional_join
			{
				// With OPTIONAL we need to add the lhs row even if we failed to find
				// any compatible rhs rows.
				info!("   optional: %s", solution_row_to_str(store, lhs));
				vec::push(result, filter_row(names, lhs));
			}
		}
	}
	
	if result.is_empty()
	{
		info!("   empty result");
	}
	
	return Solution {namespaces: copy store.namespaces, rows: result};
}

fn filter_row(names: ~[~str], row: SolutionRow) -> SolutionRow
{
	if names == ~[~"*"]
	{
		row
	}
	else
	{
		do vec::filter(row) |e| {vec::contains(names, e.first())}
	}
}

fn equal_objects(actual: Object, expected: Object) -> bool
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

fn match_subject(actual: ~str, pattern: Pattern) -> Match
{
	match pattern
	{
		Variable(name) =>
		{
			let value =
				if actual.starts_with("_:")
				{
					BlankValue(actual)
				}
				else
				{
					IriValue(actual)
				};
			either::Left({name: name, value: value})
		}
		Constant(IriValue(value)) =>
		{
			let matched = actual == value;
			//debug!("Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			either::Right(matched)
		}
		Constant(BlankValue(value)) =>
		{
			let matched = actual == value;
			//debug!("Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			either::Right(matched)
		}
		_ =>
		{
			either::Right(false)
		}
	}
}

fn match_predicate(actual: ~str, pattern: Pattern) -> Match
{
	match pattern
	{
		Variable(name) =>
		{
			let value = IriValue(actual);
			either::Left({name: name, value: value})
		}
		Constant(IriValue(value)) =>
		{
			let matched = actual == value;
			//debug!("Actual predicate %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], value];
			either::Right(matched)
		}
		_ =>
		{
			either::Right(false)
		}
	}
}

fn match_object(actual: Object, pattern: Pattern) -> Match
{
	match pattern
	{
		Variable(name) =>
		{
			either::Left({name: name, value: actual})
		}
		Constant(expected) =>
		{
			let matched = equal_objects(actual, expected);
			//debug!("Actual object %? %s %?", actual.to_str(), [~"did not match", ~"matched")[matched as uint], expected.to_str()];
			either::Right(matched)
		}
	}
}

fn eval_match(&bindings: ~[(~str, Object)], m: Match) -> result::Result<bool, ~str>
{
	match m
	{
		either::Left(binding) =>
		{
			if option::is_none(bindings.search(binding.name))
			{
				//debug!("Bound %? to %s", binding.value, binding.name);
				vec::push(bindings, (binding.name, binding.value));
				result::Ok(true)
			}
			else
			{
				return result::Err(fmt!("Binding %s was set more than once.", binding.name));
			}
		}
		either::Right(true) =>
		{
			result::Ok(true)
		}
		either::Right(false) =>
		{
			result::Ok(false)
		}
	}
}

fn iterate_matches(store: &Store, spattern: Pattern, callback: fn (Option<Binding>, @DVec<Entry>) -> bool)
{
	fn invoke(subject: ~str, pattern: Pattern, entries: @DVec<Entry>, callback: fn (option::Option<Binding>, @DVec<Entry>) -> bool) -> bool
	{
		match match_subject(subject, pattern)
		{
			either::Left(binding) =>
			{
				callback(option::Some(binding), entries)
			}
			either::Right(true) =>
			{
				callback(option::None, entries)
			}
			either::Right(false) =>
			{
				false
			}
		}
	}
	
	match spattern
	{
		Constant(IriValue(subject)) | Constant(BlankValue(subject)) =>
		{
			// Optimization for a common case where we are attempting to match a specific subject.
			let candidate = store.subjects.find(@subject);
			if option::is_some(candidate)
			{
				info!("--- matched subject %?", subject);
				let entries = option::get(candidate);
				if !invoke(subject, spattern, entries, callback)
				{
					return;
				}
			}
		}
		_ =>
		{
			for store.subjects.each()
			|subject, entries|
			{
				debug!("--- trying subject %?", subject);
				if !invoke(*subject, spattern, entries, callback)
				{
					return;
				}
			};
		}
	}
}

// Returns the named bindings.
fn eval_basic(store: &Store, names: ~[~str], matcher: TriplePattern) -> result::Result<Solution, ~str>
{
	let mut rows = Solution {namespaces: copy store.namespaces, rows: ~[]};
	
	// Iterate over the matching subjects,
	for iterate_matches(store, matcher.subject)
	|sbinding, entries|
	{
		for (*entries).each()
		|entry|
		{
			// initialize row,
			let mut bindings = ~[];
			if option::is_some(sbinding)
			{
				//debug!("Bound %? to %s", option::get(sbinding).value, option::get(sbinding).name);
				vec::push(bindings, (option::get(sbinding).name, option::get(sbinding).value));
			}
			
			// match an entry,
			let result = do eval_match(bindings, match_predicate(entry.predicate, matcher.predicate)).chain
			|matched|
			{
				if matched
				{
					eval_match(bindings, match_object(entry.object, matcher.object))
				}
				else
				{
					result::Ok(false)
				}
			};
			
			// handle the results of matching the triple.
			match result
			{
				result::Ok(true) =>
				{
					if sbinding.is_some()
					{
						info!("basic %s matched (%s, %s, %s)", triple_pattern_to_str(store, matcher), sbinding.get().value.to_friendly_str(store.namespaces), contract_uri(store.namespaces, entry.predicate), entry.object.to_friendly_str(store.namespaces));
					}
					else
					{
						info!("basic %s matched (*, %s, %s)", triple_pattern_to_str(store, matcher), contract_uri(store.namespaces, entry.predicate), entry.object.to_friendly_str(store.namespaces));
					}
					vec::push(rows.rows, filter_row(names, bindings));
				}
				result::Ok(false) =>
				{
					// match failed: try next entry
				}
				result::Err(mesg) =>
				{
					return result::Err(mesg)
				}
			}
		}
	};
	
	result::Ok(rows)
}

fn filter_solution(context: QueryContext, names: ~[~str], solution: Solution, expr: Expr) -> result::Result<Solution, ~str>
{
	let mut result = ~[];
	vec::reserve(result, vec::len(solution.rows));
	
	for vec::each(solution.rows)
	|row|
	{
		let value = eval_expr(context, row, expr);
		match get_ebv(value)
		{
			result::Ok(true) =>
			{
				vec::push(result, filter_row(names, row));
			}
			result::Ok(false) =>
			{
				debug!("FILTER rejected %?", row);
			}
			result::Err(err) =>
			{
				return result::Err(err);
			}
		}
	}
	
	return result::Ok(Solution {namespaces: copy solution.namespaces, rows: result});
}

fn bind_solution(context: QueryContext, names: ~[~str], solution: Solution, expr: Expr, name: ~str) -> result::Result<Solution, ~str>
{
	let mut result = ~[];
	vec::reserve(result, vec::len(solution.rows));
	
	for vec::each(solution.rows)
	|row|
	{
		let value = eval_expr(context, row, expr);
		match value
		{
			UnboundValue(name) =>
			{
				return result::Err(fmt!("?%s was not bound", name));
			}
			InvalidValue(literal, kind) =>
			{
				return result::Err(fmt!("?%s is not a valid %s", literal, kind));
			}
			ErrorValue(mesg) =>
			{
				return result::Err(mesg);
			}
			_ =>
			{
				vec::push(result, filter_row(names, row + ~[(name, value)]));
			}
		}
	}
	
	return result::Ok(Solution {namespaces: copy solution.namespaces, rows: result});
}

fn eval_group(store: &Store, context: QueryContext, in_names: ~[~str], terms: ~[@Algebra]) -> result::Result<Solution, ~str>
{
	let mut result = Solution {namespaces: copy store.namespaces, rows: ~[]};
	
	for vec::eachi(terms)
	|i, term|
	{
		info!(" ");
		// We can't filter out bindings not in names until we've finished joining bindings.
		let names =
			if i == vec::len(terms) - 1
			{
				in_names
			}
			else
			{
				~[~"*"]
			};
		match term
		{
			@Filter(expr) =>
			{
				match filter_solution(context, names, result, expr)
				{
					result::Ok(solution) =>
					{
						info!("term%? %s matched %s", i, algebra_to_str(store, term), solution_to_str(store, solution));
						result = solution;
					}
					result::Err(mesg) =>
					{
						return result::Err(mesg);
					}
				}
			}
			@Bind(expr, name) =>
			{
				match bind_solution(context, names, result, expr, name)
				{
					result::Ok(solution) =>
					{
						info!("term%? %s matched %s", i, algebra_to_str(store, term), solution_to_str(store, solution));
						result = solution;
					}
					result::Err(mesg) =>
					{
						return result::Err(mesg);
					}
				}
			}
			_ =>
			{
				match eval_algebra(store, ~[~"*"], {algebra: *term ,.. context})
				{
					result::Ok(solution) =>
					{
						match *term
						{
							Optional(_t) =>
							{
								if result.rows.is_not_empty()
								{
									result = join_solutions(store, names, result, solution, true);
									info!("term%? %s matched %s", i, algebra_to_str(store, term), solution_to_str(store, result));
								}
							}
							_ =>
							{
								if solution.rows.is_empty()
								{
									info!("term%? %s matched nothing", i, algebra_to_str(store, term));
									return result::Ok(Solution {namespaces : copy store.namespaces, rows: ~[]});
								}
								else if result.rows.is_not_empty()
								{
									result = join_solutions(store, names, result, solution, false);
									info!("term%? %s matched %s", i, algebra_to_str(store, term), solution_to_str(store, result));
								}
								else if i == 0		// the very first pattern in the group has nothing to join with
								{
									result = solution;
									info!("term%? %s matched %s", i, algebra_to_str(store, term), solution_to_str(store, result));
								}
							}
						}
					}
					result::Err(mesg) =>
					{
						return result::Err(mesg);
					}
				}
			}
		}
	}
	
	return result::Ok(result);
}

fn eval_optional(store: &Store, names: ~[~str], context: QueryContext, term: Algebra) -> result::Result<Solution, ~str>
{
	match eval_algebra(store, names, {algebra: term, ..context})
	{
		result::Ok(solution) =>
		{
			result::Ok(solution)
		}
		result::Err(_mesg) =>
		{
			result::Ok(Solution {namespaces: copy store.namespaces, rows: ~[]})
		}
	}
}

fn eval_algebra(store: &Store, names: ~[~str], context: QueryContext) -> result::Result<Solution, ~str>
{
	match context.algebra
	{
		Basic(pattern) =>
		{
			eval_basic(store, names, pattern)
		}
		Group(terms) =>
		{
			eval_group(store, context, names, terms)
		}
		Optional(term) =>
		{
			eval_optional(store, names, context, *term)
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

fn eval_order_expr(context: QueryContext, row: SolutionRow, expr: Expr) -> (bool, Object)
{
	match expr
	{
		CallExpr(~"!desc", e) =>
		{
			(false, eval_expr(context, row, *e[0])) 
		}
		CallExpr(~"!asc", e) =>
		{
			(true, eval_expr(context, row, *e[0]))
		}
		_ =>
		{
			(true, eval_expr(context, row, expr))
		}
	}
}

fn compare_order_values(lhs: (bool, Object), rhs: (bool, Object)) -> result::Result<int, ~str>
{
	assert lhs.first() == rhs.first();
	
	match lhs
	{
		(true, x) =>
		{
			compare_values(~"<", x, rhs.second())		// ascending
		}
		(false, x) =>
		{
			compare_values(~"<", rhs.second(), x)		// descending
		}
	}
}

fn order_by(context: QueryContext, solution: Solution, ordering: ~[Expr]) -> result::Result<Solution, ~str>
{
	// TODO
	// Probably more efficient to do the evaluation in a pre-pass. Looks like rust requires 2N comparisons in the worst case.
	// http://www.codecodex.com/wiki/Merge_sort#Analysis
	// Or maybe just do an in place sort.
	pure fn compare_rows(err_mesg: @mut ~str, ordering: ~[Expr], context: QueryContext, row1: SolutionRow, row2: SolutionRow) -> bool
	{
		unchecked
		{
			let order1 = vec::map(ordering, |o| {eval_order_expr(context, row1, o)});
			let order2 = vec::map(ordering, |o| {eval_order_expr(context, row2, o)});
			let order = vec::map2(order1, order2, |x, y| {compare_order_values(x, y)});
			let order = do vec::foldl(result::Ok(0), order)
			|x, y|
			{
				match x
				{
					result::Ok(0)	=>	y,
					_			 	=> x,
				}
			};
			match order
			{
				result::Ok(x) =>
				{
					x < 0
				}
				result::Err(err) =>
				{
					if str::is_empty(*err_mesg)
					{
						*err_mesg = err;
					}
					false
				}
			}
		}
	}
	
	let err_mesg = @mut ~"";
	
	let rows = std::sort::merge_sort(|x, y| {compare_rows(err_mesg, ordering, context, *x, *y)}, solution.rows);	// TODO: probably dont want to de-reference the pointers
	if str::is_empty(*err_mesg)
	{
		result::Ok(Solution {namespaces: copy solution.namespaces, rows: rows})
	}
	else
	{
		result::Err(*err_mesg)
	}
}

fn make_distinct(solution: Solution) -> result::Result<Solution, ~str>
{
	// TODO: Could skip this, but only if the user uses ORDER BY for every variable in the result.
	let rows = std::sort::merge_sort(|x, y| {*x < *y}, solution.rows);	// TODO: probably dont want to de-reference the pointers
	
	let mut result = ~[];
	vec::reserve(result, vec::len(rows));
	
	let mut i = 0;
	while i < vec::len(rows)
	{
		let row = rows[i];
		vec::push(result, row);
		
		i = i + 1;
		while i < vec::len(rows) && row == rows[i]
		{
			i += 1;
		}
	}
	
	return result::Ok(Solution {namespaces: copy solution.namespaces, rows: result});
}

fn eval(names: ~[~str], context: QueryContext) -> Selector
{
	let names = names;
	let context = copy context;
	|store: &Store| 
	{
		info!("algebra: %s", algebra_to_str(store, &context.algebra));
		let context = {namespaces: @store.namespaces, extensions: @store.extensions, ..context};
		do eval_algebra(store, names, context).chain()
		|solution|
		{
			// Optionally remove duplicates.
			do result::chain(if context.distinct {make_distinct(solution)} else {result::Ok(solution)})
			|solution|
			{
				// Optionally sort the solution.
				do result::chain(if vec::is_not_empty(context.order_by) {order_by(context, solution, context.order_by)} else {result::Ok(solution)})
				|solution|
				{
					match context.limit
					{
						// Optionally limit the solution.
						option::Some(limit) if limit < vec::len(solution.rows) =>
						{
							result::Ok(Solution {namespaces: solution.namespaces, rows: vec::slice(solution.rows, 0, limit)})
						}
						_ =>
						{
							result::Ok(solution)
						}
					}
				}
			}
		}
	}
}
