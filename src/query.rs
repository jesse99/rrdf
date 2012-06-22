// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;
import result::extensions;
import std::time::tm;
import sparql::*;

export eval_bg_pair, eval, pattern;

type match = either::either<binding, bool>;	// match succeeded if bindings or true

enum pattern
{
	variable(str),
	constant(object)
}

//type solutions = [solution];	// result of one or more triple patterns against the store (potential solutions that must be unified)

// Conceptually treats solution_row as a set where each set value consists of both
// the name and the value. Takes the cross product of entries from each pair
// of groups and adds compatible results to the result.
//
// Where a cross product is compatible if, for every identical name, the values
// are also identical.
fn eval_bg_pair(group1: solution, group2: solution) -> solution
{
	fn compatible_binding(b1: binding, rhs: solution_row) -> bool
	{
		alt vec::find(rhs, {|b2| b1.name == b2.name})
		{
			option::some(v2)
			{
				rdf_term_equal(b1.value, v2.value)
			}
			option::none()
			{
				true
			}
		}
	}
	
	fn compatible_row(row: solution_row, rhs: solution_row) -> bool
	{
		for vec::each(row)
		{|binding|
			if !compatible_binding(binding, rhs)
			{
				ret false;
			}
		}
		ret true;
	}
	
	fn union_rows(lhs: solution_row, rhs: solution_row) -> solution_row
	{
		let mut result = copy(lhs);
		
		for vec::each(rhs)
		{|binding2|
			alt vec::find(lhs, {|binding1| binding1.name == binding2.name})
			{
				option::some(_)
				{
					// Binding2 should be compatible with lhs so nothing to do here.
				}
				option::none()
				{
					// This is a binding in rhs but not lhs, so we need to add it to the result.
					vec::push(result, binding2);
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
			for vec::each(group2)
			{|rhs|
				#debug["testing %? and %?", lhs, rhs];
				if compatible_row(lhs, rhs)
				{
					#debug["   adding %? to result", union_rows(lhs, rhs)];
					vec::push(result, union_rows(lhs, rhs));
				}
				else
				{
					#debug["   not compatible"];
				}
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

// See 17.4.1.7 
fn rdf_term_equal(actual: object, expected: object) -> bool
{
	actual.lang == expected.lang && actual.kind == expected.kind && actual.value == expected.value
}

fn match_subject(actual: str, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			let value =
				if actual.starts_with("{")
				{
					{value: actual, kind: "blank", lang: ""}
				}
				else
				{
					{value: actual, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}
				};
			either::left({name: name, value: value})
		}
		constant({value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""})
		{
			let matched = actual == value;
			#debug["Actual subject %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], value];
			either::right(matched)
		}
		constant({value: value, kind: "blank", lang: ""})
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
			let value = {value: actual, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""};
			either::left({name: name, value: value})
		}
		constant({value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""})
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
			let matched = rdf_term_equal(actual, expected);
			#debug["Actual object %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], expected.to_str()];
			either::right(matched)
		}
	}
}

fn eval_match(context: hashmap<str, object>, match: match) -> result::result<bool, str>
{
	alt match
	{
		either::left(binding)
		{
			let new_key = context.insert(binding.name, binding.value);
			if new_key
			{
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
		constant({value: subject, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}) |
		constant({value: subject, kind: "blank", lang: ""})
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

// Returns the named bindings. Binding values for names not 
// returned by the matcher are set to none. TODO: is that right?
fn eval_bp(store: store, names: [str], matcher: triple_pattern) -> result::result<solution, str>
{
	let mut rows: solution = [];
	
	// Iterate over the matching subjects,
	for iterate_matches(store, matcher.subject)
	{|sbinding, entries|
		for (*entries).each()
		{|entry|
			// initialize context,
			let context = std::map::str_hash();
			if option::is_some(sbinding)
			{
				context.insert(option::get(sbinding).name, option::get(sbinding).value);
			}
			
			// match an entry,
			let result = eval_match(context, match_predicate(entry.predicate, matcher.predicate)).chain
			{|matched|
				if matched
				{
					eval_match(context, match_object(entry.object, matcher.object))
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
					let row = vec::map(names, {|name| {name: name, value: context.get(name)}});
					vec::push(rows, row);
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

fn solution_to_str(solution: solution) -> str
{
	let mut result = "";
	
	for vec::each(solution)
	{|row|
		let bindings = vec::map(row, {|b| #fmt["%s = %s", b.name, b.value.to_str()]});
		result += #fmt["%s\n", str::connect(bindings, ", ")];
	};
	
	ret result;
}

fn eval_bgp(store: store, names: [str], patterns: [triple_pattern]) -> result::result<solution, str>
{
	let mut result = [];
	
	for vec::each(patterns)
	{|pattern|
		alt eval_bp(store, names, pattern)
		{
			result::ok(solution)
			{
				result = eval_bg_pair(result, solution);
			}
			result::err(mesg)
			{
				ret result::err(mesg);
			}
		}
	}
	
	ret result::ok(result);
}

fn eval(names: [str], matcher: algebra) -> selector
{
	{|store: store|
		alt matcher
		{
			bp(pattern)
			{
				eval_bp(store, names, pattern)
			}
			bgp(patterns)
			{
				eval_bgp(store, names, patterns)
			}
		}
	}
}
