// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;
import result::extensions;
import std::time::tm;
import operators::*;
import sparql::*;

export join_solutions, eval, selector, pattern;

type binding = {name: str, value: object};

type match = either::either<binding, bool>;	// match succeeded if bindings or true

enum pattern
{
	variable(str),
	constant(object)
}

#[doc = "The function returned by compile and invoked to execute a SPARQL query.

Returns a solution or a 'runtime' error."]
type selector = fn@ (store) -> result::result<solution, str>;

fn solution_row_to_str(row: solution_row) -> str
{
	let mut entries = [];
	for row.each {|entry| vec::push(entries, #fmt["%s: %s", tuple::first(entry), tuple::second(entry).to_str()])};
	str::connect(entries, ", ")
}

fn solution_to_str(solution: solution) -> str
{
	let mut result = "";
	
	for vec::each(solution)
	{|row|
		result += solution_row_to_str(row);
		result += "\n";
	};
	
	ret result;
}

// Conceptually treats solution_row as a set where each set value consists of both
// the name and the value. Takes the cross product of entries from each pair
// of groups and adds compatible results to the result.
//
// Where a cross product is compatible if, for every identical name, the values
// are also identical.
fn join_solutions(names: [str], group1: solution, group2: solution, optional_join: bool) -> solution
{
	fn compatible_binding(name1: str, value1: object, rhs: solution_row) -> bool
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
		{|entry|
			if !compatible_binding(tuple::first(entry), tuple::second(entry), rhs)
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
		{|entry2|
			alt lhs.search(tuple::first(entry2))
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
	
	let mut result = [];
	if vec::is_not_empty(group1) && vec::is_not_empty(group2)
	{
		for vec::each(group1)
		{|lhs|
			let count = vec::len(result);
			for vec::each(group2)
			{|rhs|
				#debug["testing [%s] and [%s]", solution_row_to_str(lhs), solution_row_to_str(rhs)];
				if compatible_row(lhs, rhs)
				{
					let unioned = union_rows(lhs, rhs);
					#debug["   adding [%s] to result", solution_row_to_str(unioned)];
					vec::push(result, filter_row(names, unioned));
				}
				else
				{
					#debug["   not compatible"];
				}
			}
			if vec::len(result) == count && optional_join
			{
				// With OPTIONAL we need to add the lhs row even if we failed to find
				// any compatible rhs rows.
				vec::push(result, filter_row(names, lhs));
			}
		}
	}
	else if vec::is_not_empty(group1)
	{
		result = group1;
	}
	else if vec::is_not_empty(group2)
	{
		result = group2;
	}
	
	ret result;
}

fn filter_row(names: [str], row: solution_row) -> solution_row
{
	if names == ["*"]
	{
		row
	}
	else
	{
		vec::filter(row) {|e| vec::contains(names, tuple::first(e))}
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

fn match_subject(actual: str, pattern: pattern) -> match
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
			#debug["Actual subject %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], value];
			either::right(matched)
		}
		constant(blank_value(value))
		{
			let matched = actual == value;
			#debug["Actual subject %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], value];
			either::right(matched)
		}
		_
		{
			either::right(false)
		}
	}
}

fn match_predicate(actual: str, pattern: pattern) -> match
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
			#debug["Actual predicate %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], value];
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
			#debug["Actual object %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], expected.to_str()];
			either::right(matched)
		}
	}
}

fn eval_match(&bindings: [(str, object)], match: match) -> result::result<bool, str>
{
	alt match
	{
		either::left(binding)
		{
			if option::is_none(bindings.search(binding.name))
			{
				#debug["Bound %? to %s", binding.value, binding.name];
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
	fn invoke(subject: str, pattern: pattern, entries: @dvec<entry>, callback: fn (option<binding>, @dvec<entry>) -> bool) -> bool
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
				#debug["--- matched subject %?", subject];
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
			{|subject, entries|
				#debug["--- matched subject %?", subject];
				if !invoke(subject, spattern, entries, callback)
				{
					ret;
				}
			};
		}
	}
}

// Returns the named bindings.
fn eval_basic(store: store, names: [str], matcher: triple_pattern) -> result::result<solution, str>
{
	let mut rows: solution = [];
	
	// Iterate over the matching subjects,
	for iterate_matches(store, matcher.subject)
	{|sbinding, entries|
		for (*entries).each()
		{|entry|
			// initialize row,
			let mut bindings = [];
			if option::is_some(sbinding)
			{
				#debug["Bound %? to %s", option::get(sbinding).value, option::get(sbinding).name];
				vec::push(bindings, (option::get(sbinding).name, option::get(sbinding).value));
			}
			
			// match an entry,
			let result = eval_match(bindings, match_predicate(entry.predicate, matcher.predicate)).chain
			{|matched|
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

fn eval_group(store: store, in_names: [str], terms: [@algebra]) -> result::result<solution, str>
{
	let mut result = [];
	
	for vec::eachi(terms)
	{|i, term|
		alt eval_algebra(store, ["*"], *term)
		{
			result::ok(solution)
			{
				// We can't filter out bindings not in names until we've finished joining bindings.
				let names =
					if i == vec::len(terms) - 1u
					{
						in_names
					}
					else
					{
						["*"]
					};
				result = join_solutions(names, result, solution, alt *term {optional(_t) {true} _ {false}});
			}
			result::err(mesg)
			{
				ret result::err(mesg);
			}
		}
	}
	
	ret result::ok(result);
}

fn eval_optional(store: store, names: [str], term: algebra) -> result::result<solution, str>
{
	alt eval_algebra(store, names, term)
	{
		result::ok(solution)
		{
			result::ok(solution)
		}
		result::err(_mesg)
		{
			result::ok([])
		}
	}
}

fn eval_algebra(store: store, names: [str], algebra: algebra) -> result::result<solution, str>
{
	alt algebra
	{
		basic(pattern)
		{
			eval_basic(store, names, pattern)
		}
		group(terms)
		{
			eval_group(store, names, terms)
		}
		optional(term)
		{
			eval_optional(store, names, *term)
		}
	}
}

fn eval(names: [str], matcher: algebra) -> selector
{
	{|store: store|
		#debug["algebra: %?", matcher];
		eval_algebra(store, names, matcher)
	}
}
