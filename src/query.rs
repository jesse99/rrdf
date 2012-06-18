// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;
import result::extensions;
import std::time::tm;
import sparql::*;

type binding = {name: str, value: iobject};
type match = either::either<binding, bool>;					// match succeeded if bindings or true

enum pattern
{
	variable(str),
	constant(iobject)
}

type solution_row = [binding];				// result of a single triple pattern match against one triple (a potential row in a solution)
type solution_group = [solution_row];		// result of a single triple pattern against a store (a potential solution)
type solution_groups = [solution_group];	// result of one or more triple patterns against the store (potential solutions that must be unified)

// Conceptually treats solution_row as a set where each set value consists of both
// the name and the value. Takes the cross product of entries from each pair
// of groups and adds compatible results to the result.
//
// Where a cross product is compatible if, for every identical name, the values
// are also identical.
fn eval_bgp(store: store, groups: solution_groups) -> solution_group		// TODO: need to use a vector pointer or a slice or something
{
	fn compatible_binding(store: store, b1: binding, rhs: solution_row) -> bool
	{
		alt vec::find(rhs, {|b2| b1.name == b2.name})
		{
			option::some(v2)
			{
				rdf_term_equal(store, b1.value, v2.value)
			}
			option::none()
			{
				true
			}
		}
	}
	
	fn compatible_row(store: store, row: solution_row, rhs: solution_row) -> bool
	{
		for vec::each(row)
		{|binding|
			if !compatible_binding(store, binding, rhs)
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
	
	fn unify(store: store, group1: solution_group, group2: solution_group) -> solution_group
	{
		let mut result = [];
		
		if vec::is_not_empty(group1) && vec::is_not_empty(group2)
		{
			for vec::each(group1)
			{|lhs|
				for vec::each(group2)
				{|rhs|
					#debug["testing %? and %?", lhs, rhs];
					if compatible_row(store, lhs, rhs)
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
	
	let mut result = [];
	
	#debug["------ evaluating %? ------", groups];
	for vec::each(groups)
	{|group|
		result = unify(store, result, group);
	}
	
	ret result;
}

// See 17.4.1.7 
fn rdf_term_equal(store: store, actual: iobject, expected: iobject) -> bool
{
	alt actual
	{
		ireference(actual_val)
		{
			alt expected
			{
				ireference(expected_val)
				{
					actual_val == expected_val		// TODO: need to % escape bogus characters (after converting to utf-8)
				}
				ityped(expected_val, {nindex: 2u, name: "anyURI"})
				{
					get_full_name(store, actual_val) == expected_val
				}
				_
				{
					false
				}
			}
		}
		ityped(actual_val, {nindex: 2u, name: "anyURI"})
		{
			alt expected
			{
				ireference(expected_val)
				{
					actual_val == get_full_name(store, expected_val)
				}
				ityped(expected_val, {nindex: 2u, name: "anyURI"})
				{
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		ityped(actual_val, actual_kind)
		{
			alt expected
			{
				ityped(expected_val, expected_kind)
				{
					actual_kind == expected_kind && actual_val == expected_val
				}
				istring(expected_val, expected_kind, "")
				{
					actual_kind == expected_kind && actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		istring(actual_val, actual_kind, "")
		{
			alt expected
			{
				istring(expected_val, expected_kind, "")
				{
					actual_kind == expected_kind && actual_val == expected_val
				}
				ityped(expected_val, expected_kind)
				{
					actual_kind == expected_kind && actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		istring(actual_val, actual_kind, actual_lang)
		{
			alt expected
			{
				istring(expected_val, expected_kind, expected_lang)
				{
					actual_lang == expected_lang && actual_kind == expected_kind && actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
	}
}

fn match_float(lhs: str, rhs: str) -> bool
{
	str::as_c_str(lhs)
	{|lptr|
		str::as_c_str(rhs)
		{|rptr|
			let lhs = libc::strtod(lptr, ptr::null());	// TODO: use f64::from_str when it is added
			let rhs = libc::strtod(rptr, ptr::null());
			lhs == rhs
		}
	}
}

// Note that the inverse is apparently the rfc3339 method.
fn parse_dateTime(literal: str) -> option<std::time::timespec>
{
	// Time zone expressed as an offset from GMT, e.g. -05:00 for EST.
	alt std::time::strptime(literal, "%FT%T%z").chain_err
		{|_err1|
			// Time zone expressed as a name, e.g. EST (technically only Z is supposed to be allowed).
			std::time::strptime(literal, "%FT%T%Z").chain_err
			{|_err2|
				// No time zone (so the time will be considered to be in the local time zone).
				std::time::strptime(literal, "%FT%T")
			}}
	{
		result::ok(time)
		{
			option::some(time.to_timespec())
		}
		result::err(_)
		{
			#error["'%s' is not an ISO 8601 dateTime", literal];
			option::none
		}
	}
}

fn match_dateTime(lhs: str, rhs: str) -> bool
{
	let lt = parse_dateTime(lhs);
	let rt = parse_dateTime(rhs); 
	if option::is_some(lt) && option::is_some(rt)
	{
		#info["left:  %?", lt];
		#info["right: %?", rt];
		option::get(lt) == option::get(rt)
	}
	else
	{
		false
	}
}

fn match_typed_literal(actual_val: str, actual_type: str, expected_val: str, expected_type: str) -> bool
{
	alt actual_type
	{
		"xsd:float" | "xsd:double"
		{
			alt expected_type
			{
				"xsd:float" | "xsd:double" | "xsd:decimal" |
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					// The same float can appear in different formats (eg 10 and 1e10) so
					// we need to compare them as floats.
					match_float(actual_val, expected_val)
				}
				_
				{
					false
				}
			}
		}
		"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte"
		{
			// sparql doesn't have any int conversion operators so we need to be prepared to match
			// different types.
			alt expected_type
			{
				"xsd:float" | "xsd:double" | "xsd:decimal"
				{
					match_float(actual_val, expected_val)
				}
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					// Barring silliness like 099 decimal integers can only be written one
					// way so comparing using strings should be OK.
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
		{
			alt expected_type
			{
				"xsd:float" | "xsd:double" | "xsd:decimal"
				{
					match_float(actual_val, expected_val)
				}
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		"xsd:dateTime"
		{
			alt expected_type
			{
				"xsd:dateTime"
				{
					// dateTime literals may represent the same date but appear differently
					// (e.g. if they use different time zones).
					match_dateTime(actual_val, expected_val)
				}
				_
				{
					false
				}
			}
		}
		_
		{
			actual_type == expected_type && actual_val == expected_val
		}
	}
}

fn match_values(store: store, actual: iobject, expected: iobject) -> bool
{
	alt actual
	{
		ireference(actual_val)
		{
			alt expected
			{
				ireference(expected_val)
				{
					actual_val == expected_val		// TODO: may need to do some sort of special URI equality test here (and for anyURI)
				}
				ityped(expected_val, {nindex: 2u, name: "anyURI"})
				{
					get_full_name(store, actual_val) == expected_val
				}
				_
				{
					false
				}
			}
		}
		ityped(actual_val, {nindex: 2u, name: "anyURI"})
		{
			alt expected
			{
				ireference(expected_val)
				{
					actual_val == get_full_name(store, expected_val)
				}
				ityped(expected_val, {nindex: 2u, name: "anyURI"})
				{
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		ityped(actual_val, {nindex: 2u, name: "string"})
		{
			alt expected
			{
				ityped(expected_val, {nindex: 2u, name: "string"})
				{
					actual_val == expected_val
				}
				istring(expected_val, _, "")
				{
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		ityped(actual_val, lkind)
		{
			alt expected
			{
				ityped(expected_val, rkind)
				{
					match_typed_literal(actual_val, get_friendly_name(store, lkind), expected_val, get_friendly_name(store, rkind))
				}
				_
				{
					false
				}
			}
		}
		istring(actual_val, _, "")
		{
			alt expected
			{
				istring(expected_val, _, "")
				{
					actual_val == expected_val
				}
				ityped(expected_val, {nindex: 2u, name: "string"})
				{
					actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
		istring(actual_val, _, llang)
		{
			alt expected
			{
				istring(expected_val, _, rlang)
				{
					llang == rlang && actual_val == expected_val
				}
				_
				{
					false
				}
			}
		}
	}
}

fn match_pattern(store: store, actual: iobject, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			either::left({name: name, value: actual})
		}
		constant(expected)
		{
			let matched = match_values(store, actual, expected);
			#debug["Actual %? %s %?", actual.to_str(), ["did not match", "matched"][matched as uint], expected.to_str()];
			either::right(matched)
		}
	}
}

fn io_rows_to_orows(store: store, rows: [[option<iobject>]]) -> [[option<object>]]
{
	vec::map(rows, {|r| 
		vec::map(r,
		{|s|
			alt s
			{
				option::some(io)
				{
					option::some(make_object(store, io))
				}
				option::none
				{
					option::none
				}
			}
		})})
}

fn eval_match(context: hashmap<str, iobject>, match: match) -> result::result<bool, str>
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

fn context_to_row(names: [str], context: hashmap<str, iobject>) -> [option<iobject>]
{
	let row: [mut option<iobject>] = vec::to_mut(vec::from_elem(vec::len(names), option::none));
	
	for context.each
	{|name, value|
		alt vec::position_elem(names, name)
		{
			option::some(index)
			{
				row[index] = option::some(value);
			}
			option::none
			{
			}
		}
	};
	
	ret vec::from_mut(row);	// TODO: may be able to speed up the vector conversions using unsafe functions
}

fn iterate_matches(store: store, spattern: pattern, callback: fn (option<binding>, @dvec<entry>) -> bool)
{
	fn invoke(store: store, subject: qname, pattern: pattern, entries: @dvec<entry>, callback: fn (option<binding>, @dvec<entry>) -> bool) -> bool
	{
		alt match_pattern(store, ireference(subject), pattern)
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
		constant(ireference(subject))
		{
			// Optimization for a common case where we are attempting to match a specific subject.
			let candidate = store.subjects.find(subject);
			if option::is_some(candidate)
			{
				#debug["--- matched subject %?", subject];
				let entries = option::get(candidate);
				if !invoke(store, subject, spattern, entries, callback)
				{
					ret;
				}
			}
		}
		constant(ityped(name, {nindex: 2u, name: "anyURI"}))
		{
			// Same as above (though we should seldom hit this version).
			let subject = make_qname(store, name);
			let candidate = store.subjects.find(subject);
			if option::is_some(candidate)
			{
				#debug["--- matched subject %?", subject];
				let entries = option::get(candidate);
				if !invoke(store, subject, spattern, entries, callback)
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
				if !invoke(store, subject, spattern, entries, callback)
				{
					ret;
				}
			};
		}
	}
}

fn executable_pattern(store: store, cp: compiled_pattern) -> pattern
{
	alt cp
	{
		variable_binding(name)
		{
			variable(name)
		}
		string_literal(value, "")
		{
			constant(ityped(value, {nindex: 2u, name: "string"}))
		}
		string_literal(value, lang)
		{
			constant(istring(value, {nindex: 2u, name: "string"}, lang))
		}
		iri_literal(value)
		{
			constant(ireference(make_qname(store, value)))
		}
		prefixed_name(name)
		{
			constant(ireference(make_qname(store, name)))
		}
		typed_literal(value, kind)
		{
			constant(ityped(value, make_qname(store, kind)))
		}
	}
}

// Returns the named bindings. Binding values for names not 
// returned by the matcher are set to none. TODO: is that right?
fn select(names: [str], matcher: compiled_triple_pattern) -> selector
{
	{|store: store|
		let mut rows: [[option<iobject>]] = [];
		
		let spattern = executable_pattern(store, matcher.subject);
		let ppattern = executable_pattern(store, matcher.predicate);
		let opattern = executable_pattern(store, matcher.object);
		
		// Iterate over the matching subjects,
		for iterate_matches(store, spattern)
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
				let result = eval_match(context, match_pattern(store, ireference(entry.predicate), ppattern)).chain
				{|matched|
					if matched
					{
						eval_match(context, match_pattern(store, entry.object, opattern))
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
						vec::push(rows, context_to_row(names, context));
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
		
		result::ok({names: names, rows: io_rows_to_orows(store, rows)})
	}
}

