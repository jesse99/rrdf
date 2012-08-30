//! Used when evaluating a SPARQL query. Clients will not ordinarily use this.

// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import expression::*;
import operators::*;

export join_solutions, eval, pattern, variable, constant, algebra, triple_pattern, query_context,
	basic, group, optional, bind, filter;

type binding = {name: ~str, value: object};

type match = either::either<binding, bool>;	// match succeeded if bindings or true

enum pattern
{
	variable(~str),
	constant(object)
}

type triple_pattern = {subject: pattern, predicate: pattern, object: pattern};

enum algebra
{
	basic(triple_pattern),
	group(~[@algebra]),
	optional(@algebra),
	bind(expr, ~str),
	filter(expr)
}

type query_context =
	{
		namespaces: ~[namespace],
		extensions: @hashmap<~str, extension_fn>,
		algebra: algebra,
		order_by: ~[expr],
		distinct: bool,
		limit: option<uint>,
		rng: rand::rng,		// for RAND
		timestamp: tm		// for NOW
	};

fn pattern_to_str(store: store, pattern: pattern) -> ~str
{
	alt pattern
	{
		variable(v)
		{
			#fmt["?%s", v]
		}
		constant(c)
		{
			object_to_str(store, c)
		}
	}
}

fn triple_pattern_to_str(store: store, pattern: triple_pattern) -> ~str
{
	#fmt["{subject: %s, predicate: %s, object: %s}", pattern_to_str(store, pattern.subject), pattern_to_str(store, pattern.predicate), pattern_to_str(store, pattern.object)]
}
	
fn algebra_to_str(store: store, algebra: algebra) -> ~str
{
	alt algebra
	{
		basic(p)
		{
			triple_pattern_to_str(store, p)
		}
		group(args)
		{
			#fmt["[%s]", str::connect(do args.map |a| {algebra_to_str(store, *a)}, ~", ")]
		}
		optional(a)
		{
			~"optional " + algebra_to_str(store, *a)
		}
		bind(e, n)
		{
			#fmt["%s = %s", n, expr_to_str(store, e)]
		}
		filter(e)
		{
			~"filter " + expr_to_str(store, e)
		}
	}
}

fn solution_row_to_str(store: store, row: solution_row) -> ~str
{
	let mut entries = ~[];
	for row.each
	|entry|
	{
		let name = entry.first();
		let value = object_to_str(store, entry.second());
		vec::push(entries, #fmt["%s: %s", name, value]);
	};
	str::connect(entries, ~", ")
}

fn solution_to_str(store: store, solution: solution) -> ~str
{
	let mut result = ~"";
	
	for vec::eachi(solution)
	|i, row|
	{
		result += #fmt["%?: %s   ", i, solution_row_to_str(store, row)];
	};
	
	if result.is_empty()
	{
		result = ~"nothing";
	}
	
	ret result;
}

// Conceptually treats solution_row as a set where each set value consists of both
// the name and the value. Takes the cross product of entries from each pair
// of groups and adds compatible results to the result.
//
// Where a cross product is compatible if, for every identical name, the values
// are also identical.
fn join_solutions(store: store, names: ~[~str], group1: solution, group2: solution, optional_join: bool) -> solution
{
	fn compatible_binding(name1: ~str, value1: object, rhs: solution_row) -> bool
	{
		alt rhs.search(name1)
		{
			option::some(value2)
			{
				equal_objects(value1, value2)
			}
			option::none()
			{
				true
			}
		}
	}
	
	fn compatible_row(row: solution_row, rhs: solution_row) -> bool
	{
		for row.each()
		|entry|
		{
			if !compatible_binding(entry.first(), entry.second(), rhs)
			{
				ret false;
			}
		}
		ret true;
	}
	
	fn union_rows(lhs: solution_row, rhs: solution_row) -> solution_row
	{
		let mut result = copy(lhs);
		
		for rhs.each()
		|entry2|
		{
			alt lhs.search(entry2.first())
			{
				option::some(_)
				{
					// Binding should be compatible with lhs so nothing to do here.
				}
				option::none()
				{
					// This is a binding in rhs but not lhs, so we need to add it to the result.
					vec::push(result, entry2);
				}
			}
		}
		
		ret result;
	}
	
	let mut result = ~[];
	#info["joining:"];
	#info["   group1 = %s", solution_to_str(store, group1)];
	#info["   group2 = %s", solution_to_str(store, group2)];
	if vec::is_not_empty(group1) && (vec::is_not_empty(group2) || optional_join)
	{
		for vec::each(group1)
		|lhs|
		{
			let count = vec::len(result);
			for vec::each(group2)
			|rhs|
			{
				if compatible_row(lhs, rhs)
				{
					let unioned = union_rows(lhs, rhs);
					#info["   added: %s", solution_row_to_str(store, unioned)];
					vec::push(result, filter_row(names, unioned));
				}
				else
				{
					#debug["   not compatible: %s and %s", solution_row_to_str(store, lhs), solution_row_to_str(store, rhs)];
				}
			}
			if vec::len(result) == count && optional_join
			{
				// With OPTIONAL we need to add the lhs row even if we failed to find
				// any compatible rhs rows.
				#info["   optional: %s", solution_row_to_str(store, lhs)];
				vec::push(result, filter_row(names, lhs));
			}
		}
	}
	
	if result.is_empty()
	{
		#info["   empty result"];
	}
	
	ret result;
}

fn filter_row(names: ~[~str], row: solution_row) -> solution_row
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

fn equal_objects(actual: object, expected: object) -> bool
{
	alt op_equals(actual, expected)	// should get bool_value or error_value
	{
		bool_value(value)
		{
			value
		}
		_
		{
			false
		}
	}
}

fn match_subject(actual: ~str, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			let value =
				if actual.starts_with("_:")
				{
					blank_value(actual)
				}
				else
				{
					iri_value(actual)
				};
			either::left({name: name, value: value})
		}
		constant(iri_value(value))
		{
			let matched = actual == value;
			//#debug["Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched"][matched as uint], value];
			either::right(matched)
		}
		constant(blank_value(value))
		{
			let matched = actual == value;
			//#debug["Actual subject %? %s %?", actual.to_str(), [~"did not match", ~"matched"][matched as uint], value];
			either::right(matched)
		}
		_
		{
			either::right(false)
		}
	}
}

fn match_predicate(actual: ~str, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			let value = iri_value(actual);
			either::left({name: name, value: value})
		}
		constant(iri_value(value))
		{
			let matched = actual == value;
			//#debug["Actual predicate %? %s %?", actual.to_str(), [~"did not match", ~"matched"][matched as uint], value];
			either::right(matched)
		}
		_
		{
			either::right(false)
		}
	}
}

fn match_object(actual: object, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			either::left({name: name, value: actual})
		}
		constant(expected)
		{
			let matched = equal_objects(actual, expected);
			//#debug["Actual object %? %s %?", actual.to_str(), [~"did not match", ~"matched"][matched as uint], expected.to_str()];
			either::right(matched)
		}
	}
}

fn eval_match(&bindings: ~[(~str, object)], match: match) -> result::result<bool, ~str>
{
	alt match
	{
		either::left(binding)
		{
			if option::is_none(bindings.search(binding.name))
			{
				//#debug["Bound %? to %s", binding.value, binding.name];
				vec::push(bindings, (binding.name, binding.value));
				result::ok(true)
			}
			else
			{
				ret result::err(#fmt["Binding %s was set more than once.", binding.name]);
			}
		}
		either::right(true)
		{
			result::ok(true)
		}
		either::right(false)
		{
			result::ok(false)
		}
	}
}

fn iterate_matches(store: store, spattern: pattern, callback: fn (option<binding>, @dvec<entry>) -> bool)
{
	fn invoke(subject: ~str, pattern: pattern, entries: @dvec<entry>, callback: fn (option<binding>, @dvec<entry>) -> bool) -> bool
	{
		alt match_subject(subject, pattern)
		{
			either::left(binding)
			{
				callback(option::some(binding), entries)
			}
			either::right(true)
			{
				callback(option::none, entries)
			}
			either::right(false)
			{
				false
			}
		}
	}
	
	alt spattern
	{
		constant(iri_value(subject)) | constant(blank_value(subject))
		{
			// Optimization for a common case where we are attempting to match a specific subject.
			let candidate = store.subjects.find(subject);
			if option::is_some(candidate)
			{
				#info["--- matched subject %?", subject];
				let entries = option::get(candidate);
				if !invoke(subject, spattern, entries, callback)
				{
					ret;
				}
			}
		}
		_
		{
			for store.subjects.each()
			|subject, entries|
			{
				#debug["--- trying subject %?", subject];
				if !invoke(subject, spattern, entries, callback)
				{
					ret;
				}
			};
		}
	}
}

// Returns the named bindings.
fn eval_basic(store: store, names: ~[~str], matcher: triple_pattern) -> result::result<solution, ~str>
{
	let mut rows: solution = ~[];
	
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
				//#debug["Bound %? to %s", option::get(sbinding).value, option::get(sbinding).name];
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
					result::ok(false)
				}
			};
			
			// handle the results of matching the triple.
			alt result
			{
				result::ok(true)
				{
					if sbinding.is_some()
					{
						#info["basic %s matched (%s, %s, %s)", triple_pattern_to_str(store, matcher), object_to_str(store, sbinding.get().value), contract_uri(store.namespaces, entry.predicate), object_to_str(store, entry.object)];
					}
					else
					{
						#info["basic %s matched (*, %s, %s)", triple_pattern_to_str(store, matcher), contract_uri(store.namespaces, entry.predicate), object_to_str(store, entry.object)];
					}
					vec::push(rows, filter_row(names, bindings));
				}
				result::ok(false)
				{
					// match failed: try next entry
				}
				result::err(mesg)
				{
					ret result::err(mesg)
				}
			}
		}
	};
	
	result::ok(rows)
}

fn filter_solution(context: query_context, names: ~[~str], solution: solution, expr: expr) -> result::result<solution, ~str>
{
	let mut result = ~[];
	vec::reserve(result, vec::len(solution));
	
	for vec::each(solution)
	|row|
	{
		let value = eval_expr(context, row, expr);
		alt get_ebv(value)
		{
			result::ok(true)
			{
				vec::push(result, filter_row(names, row));
			}
			result::ok(false)
			{
				#debug["FILTER rejected %?", row];
			}
			result::err(err)
			{
				ret result::err(err);
			}
		}
	}
	
	ret result::ok(result);
}

fn bind_solution(context: query_context, names: ~[~str], solution: solution, expr: expr, name: ~str) -> result::result<solution, ~str>
{
	let mut result = ~[];
	vec::reserve(result, vec::len(solution));
	
	for vec::each(solution)
	|row|
	{
		let value = eval_expr(context, row, expr);
		alt value
		{
			unbound_value(name)
			{
				ret result::err(#fmt["?%s was not bound", name]);
			}
			invalid_value(literal, kind)
			{
				ret result::err(#fmt["?%s is not a valid %s", literal, kind]);
			}
			error_value(mesg)
			{
				ret result::err(mesg);
			}
			_
			{
				vec::push(result, filter_row(names, row + ~[(name, value)]));
			}
		}
	}
	
	ret result::ok(result);
}

fn eval_group(store: store, context: query_context, in_names: ~[~str], terms: ~[@algebra]) -> result::result<solution, ~str>
{
	let mut result = ~[];
	
	for vec::eachi(terms)
	|i, term|
	{
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
		alt term
		{
			@filter(expr)
			{
				alt filter_solution(context, names, result, expr)
				{
					result::ok(solution)
					{
						#info["group filter %? matched %?", expr_to_str(store, expr), solution];
						result = solution;
					}
					result::err(mesg)
					{
						ret result::err(mesg);
					}
				}
			}
			@bind(expr, name)
			{
				alt bind_solution(context, names, result, expr, name)
				{
					result::ok(solution)
					{
						#info["group bind %?/%? matched %?", expr, name, solution];
						result = solution;
					}
					result::err(mesg)
					{
						ret result::err(mesg);
					}
				}
			}
			_
			{
				alt eval_algebra(store, ~[~"*"], {algebra: *term with context})
				{
					result::ok(solution)
					{
						alt *term
						{
							optional(_t)
							{
								if result.is_not_empty()
								{
									result = join_solutions(store, names, result, solution, true);
									#info["group term %s matched %s", algebra_to_str(store, *term), solution_to_str(store, result)];
								}
							}
							_
							{
								if solution.is_empty()
								{
									ret result::ok(~[]);
								}
								else if result.is_not_empty()
								{
									result = join_solutions(store, names, result, solution, false);
									#info["group term %s matched %s", algebra_to_str(store, *term), solution_to_str(store, result)];
								}
								else
								{
									result = solution;
									#info["group term %s matched %s", algebra_to_str(store, *term), solution_to_str(store, result)];
								}
							}
						}
					}
					result::err(mesg)
					{
						ret result::err(mesg);
					}
				}
			}
		}
	}
	
	ret result::ok(result);
}

fn eval_optional(store: store, names: ~[~str], context: query_context, term: algebra) -> result::result<solution, ~str>
{
	alt eval_algebra(store, names, {algebra: term with context})
	{
		result::ok(solution)
		{
			result::ok(solution)
		}
		result::err(_mesg)
		{
			result::ok(~[])
		}
	}
}

fn eval_algebra(store: store, names: ~[~str], context: query_context) -> result::result<solution, ~str>
{
	alt context.algebra
	{
		basic(pattern)
		{
			eval_basic(store, names, pattern)
		}
		group(terms)
		{
			eval_group(store, context, names, terms)
		}
		optional(term)
		{
			eval_optional(store, names, context, *term)
		}
		bind(*)
		{
			result::err(~"BIND should appear in a pattern group.")
		}
		filter(*)
		{
			// Not sure what's supposed to happen here. According to GroupGraphPatternSub a
			// group can contain just a FILTER (should be a no-op?) or a filter and then a triple
			// pattern (filter position doesn't matter?).
			result::err(~"FILTER should appear last in a pattern group.")
		}
	}
}

fn eval_order_expr(context: query_context, row: solution_row, expr: expr) -> (bool, object)
{
	alt expr
	{
		call_expr(~"!desc", e)
		{
			(false, eval_expr(context, row, *e[0]))
		}
		call_expr(~"!asc", e)
		{
			(true, eval_expr(context, row, *e[0]))
		}
		_
		{
			(true, eval_expr(context, row, expr))
		}
	}
}

fn compare_order_values(lhs: (bool, object), rhs: (bool, object)) -> result::result<int, ~str>
{
	assert lhs.first() == rhs.first();
	
	alt lhs
	{
		(true, x)
		{
			compare_values(~"<", x, rhs.second())		// ascending
		}
		(false, x)
		{
			compare_values(~"<", rhs.second(), x)		// descending
		}
	}
}

fn order_by(context: query_context, solution: solution, ordering: ~[expr]) -> result::result<solution, ~str>
{
	let mut err_mesg = ~"";
	
	let le = |&e: ~str, row1, row2|
	{
		let order1 = vec::map(ordering, |o| {eval_order_expr(context, row1, o)});
		let order2 = vec::map(ordering, |o| {eval_order_expr(context, row2, o)});
		let order = vec::map2(order1, order2, |x, y| {compare_order_values(x, y)});
		let order = do vec::foldl(result::ok(0), order)
		|x, y|
		{
			alt x
			{
				result::ok(0)	{y}
				_				{x}
			}
		};
		alt order
		{
			result::ok(x)
			{
				x < 0
			}
			result::err(err)
			{
				if str::is_empty(e)
				{
					e = err;
				}
				false
			}
		}
	};
	
	let solution = std::sort::merge_sort(|x, y| {le(err_mesg, x, y)}, solution);
	if str::is_empty(err_mesg)
	{
		result::ok(solution)
	}
	else
	{
		result::err(err_mesg)
	}
}

fn make_distinct(solution: solution) -> result::result<solution, ~str>
{
	// TODO: Could skip this, but only if the user uses ORDER BY for every variable in the result.
	let solution = std::sort::merge_sort(|x, y| {x < y}, solution);
	
	let mut result = ~[];
	vec::reserve(result, vec::len(solution));
	
	let mut i = 0;
	while i < vec::len(solution)
	{
		let row = solution[i];
		vec::push(result, row);
		
		i = i + 1;
		while i < vec::len(solution) && row == solution[i]
		{
			i += 1;
		}
	}
	
	ret result::ok(result);
}

fn eval(names: ~[~str], context: query_context) -> selector
{
	|store: store|
	{
		#info["algebra: %s", algebra_to_str(store, context.algebra)];
		let context = {namespaces: store.namespaces, extensions: store.extensions with context};
		do eval_algebra(store, names, context).chain()
		|solution|
		{
			// Optionally remove duplicates.
			do result::chain(if context.distinct {make_distinct(solution)} else {result::ok(solution)})
			|solution|
			{
				// Optionally sort the solution.
				do result::chain(if vec::is_not_empty(context.order_by) {order_by(context, solution, context.order_by)} else {result::ok(solution)})
				|solution|
				{
					alt context.limit
					{
						// Optionally limit the solution.
						option::some(limit) if limit < vec::len(solution)
						{
							result::ok(vec::slice(solution, 0, limit))
						}
						_
						{
							result::ok(solution)
						}
					}
				}
			}
		}
	}
}
