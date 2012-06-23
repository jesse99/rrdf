// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;
import result::extensions;
import std::time::tm;
import sparql::*;

export eval_bg_pair, eval, pattern;

type binding = {name: str, value: object};

type match = either::either<binding, bool>;	// match succeeded if bindings or true

enum pattern
{
	variable(str),
	constant(object)
}

fn solution_row_to_str(row: solution_row) -> str
{
	let mut entries = [];
	for row.each {|name, value| vec::push(entries, #fmt["%s: %s", name, value.to_str()])};
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
fn eval_bg_pair(group1: solution, group2: solution) -> solution
{
	fn compatible_binding(name1: str, value1: object, rhs: solution_row) -> bool
	{
		alt rhs.find(name1)
		{
			option::some(value2)
			{
				rdf_term_equal(value1, value2)
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
		{|name, value|
			if !compatible_binding(name, value, rhs)
			{
				ret false;
			}
		}
		ret true;
	}
	
	fn union_rows(lhs: solution_row, rhs: solution_row) -> solution_row
	{
		let result = std::map::str_hash();
		
		// Copy is a shallow copy so we need to copy lhs the hard way.
		for lhs.each() {|name2, value2| result.insert(name2, value2);}
		
		for rhs.each()
		{|name2, value2|
			alt lhs.find(name2)
			{
				option::some(_)
				{
					// Binding2 should be compatible with lhs so nothing to do here.
				}
				option::none()
				{
					// This is a binding in rhs but not lhs, so we need to add it to the result.
					result.insert(name2, value2);
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
				#debug["testing [%s] and [%s]", solution_row_to_str(lhs), solution_row_to_str(rhs)];
				if compatible_row(lhs, rhs)
				{
					let unioned = union_rows(lhs, rhs);
					#debug["   adding [%s] to result", solution_row_to_str(unioned)];
					vec::push(result, unioned);
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

// TODO: This is the RDF notion of equality (see 17.4.1.7). Unfortunately this is not the SPARQL 
// notion which is apparently based on the much more complex entailment goo.
fn rdf_term_equal(actual: object, expected: object) -> bool
{
	actual.kind == expected.kind && actual.value == expected.value && str::to_lower(actual.lang) == str::to_lower(expected.lang)
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

// Returns the named bindings.
fn eval_bp(store: store, matcher: triple_pattern) -> result::result<solution, str>
{
	let mut rows: solution = [];
	
	// Iterate over the matching subjects,
	for iterate_matches(store, matcher.subject)
	{|sbinding, entries|
		for (*entries).each()
		{|entry|
			// initialize row,
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
					vec::push(rows, context);
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

fn eval_bgp(store: store, patterns: [triple_pattern]) -> result::result<solution, str>
{
	let mut result = [];
	
	for vec::each(patterns)
	{|pattern|
		alt eval_bp(store, pattern)
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

fn eval(matcher: algebra) -> selector
{
	{|store: store|
		alt matcher
		{
			bp(pattern)
			{
				eval_bp(store, pattern)
			}
			bgp(patterns)
			{
				eval_bgp(store, patterns)
			}
		}
	}
}
