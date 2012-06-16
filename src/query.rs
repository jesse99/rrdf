// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;
import result::extensions;
import sparql::*;

type binding = {name: str, value: iobject};
type match = either::either<binding, bool>;					// match succeeded if bindings or true

enum pattern
{
	variable(str),
	constant(iobject)
}

fn get_match_type(object: iobject) -> str
{
	alt object
	{
		ireference(_subject)
		{
			"2:anyURI"
		}
		ityped(_value, kind)
		{
			#fmt["%?:%s", kind.nindex, kind.name] 
		}
		istring(_value, _kind, lang)
		{
			"@" + lang
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

fn match_typed_literal(lvalue: str, ltype: str, rvalue: str, rtype: str) -> bool
{
	alt ltype
	{
		"xsd:float" | "xsd:double"
		{
			alt rtype
			{
				"xsd:float" | "xsd:double" | "xsd:decimal" |
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					// The same float can appear in different formats (eg 10 and 1e10) so
					// we need to compare them as floats.
					match_float(lvalue, rvalue)
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
			alt rtype
			{
				"xsd:float" | "xsd:double" | "xsd:decimal"
				{
					match_float(lvalue, rvalue)
				}
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					// Barring silliness like 099 decimal integers can only be written one
					// way so comparing using strings should be OK.
					lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
		"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
		{
			alt rtype
			{
				"xsd:float" | "xsd:double" | "xsd:decimal"
				{
					match_float(lvalue, rvalue)
				}
				"xsd:integer" | "xsd:nonPositiveInteger" | "xsd:long" | "xsd:negativeInteger" | "xsd:int" | "xsd:short" | "xsd:byte" |
				"xsd:nonNegativeInteger" | "xsd:unsignedLong" | "xsd:unsignedInt" | "xsd:unsignedShort" | "xsd:unsignedByte" | "xsd:positiveInteger"
				{
					lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
		_
		{
			ltype == rtype && lvalue == rvalue
		}
	}
}

fn match_values(store: store, lhs: iobject, rhs: iobject) -> bool
{
	alt lhs
	{
		ireference(lvalue)
		{
			alt rhs
			{
				ireference(rvalue)
				{
					lvalue == rvalue		// TODO: may need to do some sort of special URI equality test here (and for anyURI)
				}
				ityped(rvalue, {nindex: 2u, name: "anyURI"})
				{
					get_full_name(store, lvalue) == rvalue
				}
				_
				{
					false
				}
			}
		}
		ityped(lvalue, {nindex: 2u, name: "anyURI"})
		{
			alt rhs
			{
				ireference(rvalue)
				{
					lvalue == get_full_name(store, rvalue)
				}
				ityped(rvalue, {nindex: 2u, name: "anyURI"})
				{
					lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
		ityped(lvalue, {nindex: 2u, name: "string"})
		{
			alt rhs
			{
				ityped(rvalue, {nindex: 2u, name: "string"})
				{
					lvalue == rvalue
				}
				istring(rvalue, _, "")
				{
					lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
		ityped(lvalue, lkind)
		{
			alt rhs
			{
				ityped(rvalue, rkind)
				{
					match_typed_literal(lvalue, get_friendly_name(store, lkind), rvalue, get_friendly_name(store, rkind))
				}
				_
				{
					false
				}
			}
		}
		istring(lvalue, _, "")
		{
			alt rhs
			{
				istring(rvalue, _, "")
				{
					lvalue == rvalue
				}
				ityped(rvalue, {nindex: 2u, name: "string"})
				{
					lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
		istring(lvalue, _, llang)
		{
			alt rhs
			{
				istring(rvalue, _, rlang)
				{
					llang == rlang && lvalue == rvalue
				}
				_
				{
					false
				}
			}
		}
	}
}

fn match_pattern(store: store, lhs: iobject, pattern: pattern) -> match
{
	alt pattern
	{
		variable(name)
		{
			either::left({name: name, value: lhs})
		}
		constant(rhs)
		{
			let type1 = get_match_type(lhs);
			let type2 = get_match_type(rhs);
			if type1 == type2
			{
				let matched = match_values(store, lhs, rhs);
				#debug["Values %? (%s) and %? (%s) %s", lhs.to_str(), type1, rhs.to_str(), type2, ["don't match", "match"][matched as uint]];
				either::right(matched)
			}
			else
			{
				#debug["Types '%s' and '%s' do not match", type1, type2];
				either::right(false)
			}
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

