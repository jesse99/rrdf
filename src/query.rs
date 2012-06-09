// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;

#[doc = "Names appear in the same order as the variables in the SELECT clause.

Each returned row will name (len(names) columns."]
type solution = {names: [str], rows: [[option<object>]]};

type binding = {name: str, value: option<object>};
type match = either::either<[binding], bool>;					// match succeeded if bindings or true
type matcher = fn@ (triple) -> match;

type selector = fn@ ([triple]) -> result::result<solution, str>;	// returns a solution or an error

enum pattern
{
	variable(str),
	constant(object)
}

// Probably should return xsd types here.
fn get_match_type(object: object) -> str
{
	alt object
	{
		reference(subject)
		{
			alt subject
			{
				iri(_uri)
				{
					"iri"
				}
				blank(_value)
				{
					"blank node"
				}
			}
		}
		typed_literal(_value, "xsd:string") | typed_literal(_value, "http://www.w3.org/2001/XMLSchema#string")
		{
			"string"
		}
		typed_literal(_value, klass)
		{
			klass + " literal"
		}
		plain_literal(_text, lang)
		{
			lang + " language string"
		}
		xml(_text)
		{
			"xml"
		}
		string(_text)
		{
			"string"
		}
		boolean(_value)
		{
			"boolean"
		}
		decimal(_) | integer(_) | nonPositiveInteger(_) | nonNegativeInteger(_) | long(_) | int(_) | short(_) | byte(_) | unsignedLong(_) | unsignedInt(_) | unsignedShort(_) | unsignedByte(_) | float(_) | double(_)
		{
			"number"
		}
		duration(_value)
		{
			"duration"
		}
		dateTime(_value)
		{
			"dateTime"
		}
		time(_value)
		{
			"time"
		}
		date(_value)
		{
			"date"
		}
		hexBinary(_value)			// TODO: should use one type for binary and base-64
		{
			"binary"
		}
		base64Binary(_value)
		{
			"base64"
		}
		anyURI(_value)
		{
			"iri"
		}
		language(_value)
		{
			"language"
		}
	}
}

// TODO: Typed literals aren't handled properly. For example, "3.14"^^xsd:double won't match 3.14.
fn get_match_value(object: object) -> object
{
	alt object
	{
		reference(subject)
		{
			alt subject
			{
				iri(value)
				{
					anyURI(value)
				}
				blank(_value)
				{
					object
				}
			}
		}
		typed_literal(value, "xsd:string") | typed_literal(value, "http://www.w3.org/2001/XMLSchema#string")
		{
			string(value)
		}
		decimal(value)
		{
			alt float::from_str(value)		// TODO: use a double module once it is available
			{
				option::some(value)
				{
					double(value)
				}
				option::none
				{
					object
				}
			}
		}
		integer(value) | nonPositiveInteger(value)
		{
			alt int::from_str(value)		// TODO: use a i64 module once it is available
			{
				option::some(value)
				{
					long(value)
				}
				option::none
				{
					object
				}
			}
		}
		nonNegativeInteger(value)
		{
			alt u64::from_str(value, 10u64)
			{
				option::some(value)
				{
					unsignedLong(value)
				}
				option::none
				{
					object
				}
			}
		}
		int(value)
		{
			long(value as i64)
		}
		short(value)
		{
			long(value as i64)
		}
		byte(value)
		{
			long(value as i64)
		}
		unsignedInt(value)
		{
			unsignedLong(value as u64)
		}
		unsignedShort(value)
		{
			unsignedLong(value as u64)
		}
		unsignedByte(value)
		{
			unsignedLong(value as u64)
		}
		float(value)
		{
			double(value as f64)
		}
		_
		{
			object
		}
	}
}

// TODO: need to special case anyURI (eg for namespaces and probably other stuff)
fn match_values(lhs: object, rhs: object) -> bool
{
	alt lhs
	{
		long(value1)				// these are the only three numeric types we need to worry about
		{							// TODO: until we better support arbitrary precision numbers anyway
			alt rhs
			{
				unsignedLong(value2)
				{
					if value1 > 0
					{
						(value1 as u64) == value2
					}
					else
					{
						false
					}
				}
				double(value2)
				{
					(value1 as f64) == value2
				}
				_
				{
					lhs == rhs
				}
			}
		}
		unsignedLong(value1)
		{
			alt rhs
			{
				long(value2)
				{
					if value2 > 0
					{
						value1 == (value2 as u64)
					}
					else
					{
						false
					}
				}
				double(value2)
				{
					(value1 as f64) == value2
				}
				_
				{
					lhs == rhs
				}
			}
		}
		double(value1)
		{
			alt rhs
			{
				long(value2)
				{
					value1 == (value2 as f64)
				}
				unsignedLong(value2)
				{
					value1 == (value2 as f64)
				}
				_
				{
					lhs == rhs
				}
			}
		}
		_
		{
			lhs == rhs
		}
	}
}

fn match_literal(lhs: object, rhs: object) -> match
{
	let type1 = get_match_type(lhs);
	let type2 = get_match_type(rhs);
	if type1 == type2
	{
		let value1 = get_match_value(lhs);
		let value2 = get_match_value(rhs);
		let matched = match_values(value1, value2);
		#debug["Values %? (%s) and %? (%s) %s", value1, type1, value2, type2, ["don't match", "match"][matched as uint]];
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
	{|triplet: triple|
		let s = triplet.subject;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(reference(s))};
				core::either::left([b])
			}
			constant(rhs)
			{
				match_literal(reference(s), rhs)
			}
		}
	}
}

fn match_property(pattern: pattern) -> matcher
{
	{|triplet: triple|
		let p = triplet.property;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(anyURI(p))};
				core::either::left([b])
			}
			constant(rhs)
			{
				match_literal(reference(iri(p)), rhs)
			}
		}
	}
}

fn match_object(pattern: pattern) -> matcher
{
	{|triplet: triple|
		let o = triplet.object;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(triplet.object)};
				either::left([b])
			}
			constant(rhs)
			{
				match_literal(o, rhs)
			}
		}
	}
}

// Returns the named bindings. Binding values for names not 
// returned by the matchers are set to none.
fn select(names: [str], matchers: [matcher]) -> selector
{
	{|triples: [triple]|
		let mut rows: [[option<object>]] = [];
		
		for vec::each(triples)
		{|triple|
			let row: [mut option<object>] = vec::to_mut(vec::from_elem(vec::len(names), option::none));
			let mut matched = true;
			
			for vec::each(matchers)
			{|matcher|
				alt matcher(triple)
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
		};
		
		result::ok({names: names, rows: rows})
	}
}
