// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;

type binding = {name: str, value: option<iobject>};
type match = either::either<[binding], bool>;					// match succeeded if bindings or true
type matcher = fn@ (store, qname, entry) -> match;

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
			"reference"
		}
		ityped(_value, kind)
		{
			#fmt["%?:%s", kind.nindex, kind.name] 
		}
		iplain(_value, lang)
		{
			#fmt["plain@%s", lang] 
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
		iplain(lvalue, llang)
		{
			alt rhs
			{
				iplain(rvalue, rlang)
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

fn match_literal(store: store, lhs: iobject, rhs: iobject) -> match
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

fn match_subject(pattern: pattern) -> matcher
{
	{|store: store, subject: qname, _entry: entry|
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(ireference(subject))};
				core::either::left([b])
			}
			constant(rhs)
			{
				match_literal(store, ireference(subject), rhs)
			}
		}
	}
}

fn match_predicate(pattern: pattern) -> matcher
{
	{|store: store, _subject: qname, entry: entry|
		alt pattern
		{
			variable(name)
			{
				let p = get_friendly_name(store, entry.predicate);
				let b = {name: name, value: some(ityped(p, {nindex: 2u, name: "anyURI"}))};
				core::either::left([b])
			}
			constant(rhs)
			{
				match_literal(store, ireference(entry.predicate), rhs)
			}
		}
	}
}

fn match_object(pattern: pattern) -> matcher
{
	{|store: store, _subject: qname, entry: entry|
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(entry.object)};
				either::left([b])
			}
			constant(rhs)
			{
				match_literal(store, entry.object, rhs)
			}
		}
	}
}

// Returns the named bindings. Binding values for names not 
// returned by the matchers are set to none.
fn select(names: [str], matchers: [matcher]) -> selector
{
	{|store: store|
		let mut rows: [[option<iobject>]] = [];
		
		for store.subjects.each()
		{|subject, entries|
			for (*entries).each()
			{|entry|
				let row: [mut option<iobject>] = vec::to_mut(vec::from_elem(vec::len(names), option::none));
				let mut matched = true;
				
				for vec::each(matchers)
				{|matcher|
					alt matcher(store, subject, entry)
					{
						core::either::left(bindings)
						{
							for vec::each(bindings)
							{|binding|
								alt vec::position_elem(names, binding.name)
								{
									option::some(index)
									{
										if row[index] == option::none
										{
											row[index] = binding.value;
										}
										else
										{
											// Spec isn't clear what the semantics of this should be, but it seems
											// likely to be, at best, confusing and normally a bug so we'll call it
											// an error for now.
											ret result::err(#fmt["Binding %s was set more than once.", binding.name]);
										}
									}
									option::none
									{
										// Matcher created a binding, but it's not one the user wanted returned
										// (though it could be used by other matchers).
									}
								}
							};
						}
						core::either::right(true)
						{
							// We matched the triple so we can keep going, but this
							// matcher doesn't have any bindings so there is nothing
							// to return for it.
						}
						core::either::right(false)
						{
							matched = false;
							break;
						}
					}
				};
				
				if matched
				{
					vec::push(rows, vec::from_mut(row));	// TODO: may be able to speed up the vector conversions using unsafe functions
				}
			}
		};
		
		let rs = vec::map(rows, {|r| 
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
			})});
		result::ok({names: names, rows: rs})
	}
}
