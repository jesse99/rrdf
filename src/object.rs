#[doc = "Value component of a triple and associated methods."];
import result::extensions;

// Note that the SPARQL error conditions do not always result in an error:
// 1) Functions like COALESCE accept unbound variables.
// 2) Boolean functions normally want effective boolean values which are false for invalid values.
// 3) Functions like op_and do not always propagate errors.
#[doc = "Value component of a triple."]
enum object				// TODO: once we support serialization we'll need to add something like u8 type codes to int, float, and string values
{								// TODO: predicate could maybe be enum with type code and uri
	// literals
	bool_value(bool),
	int_value(i64),			// value, xsd:decimal (and derived types)
	float_value(f64),			// value, xsd:float or xsd:double
	dateTime_value(tm),	// xsd:dateTime
	string_value(str, str),	// value + lang
	typed_value(str, str),	// value + type iri (aka simple literal)
	
	// other rdf terms
	iri_value(str),
	blank_value(str),
	
	// error conditions
	unbound_value(str),	// binding name
	invalid_value(str, str),	// literal + type iri
	error_value(str)			// err mesg
}

impl object_methods for object
{
	fn as_bool() -> bool
	{
		alt self
		{
			bool_value(value)
			{
				value
			}
			_
			{
				fail(#fmt["Expected a bool_value but found %?", self]);
			}
		}
	}
	
	fn as_int() -> int
	{
		alt self
		{
			int_value(value)
			{
				if value >= int::min_value as i64 && value <= int::max_value as i64
				{
					value as int
				}
				else
				{
					fail(#fmt["Can't convert %? to an int", self]);
				}
			}
			_
			{
				fail(#fmt["Expected an int_value but found %?", self]);
			}
		}
	}
	
	fn as_uint() -> uint
	{
		alt self
		{
			int_value(value)
			{
				if value >= uint::min_value as i64 && value <= uint::max_value as i64
				{
					value as uint
				}
				else
				{
					fail(#fmt["Can't convert %? to a uint", self]);
				}
			}
			_
			{
				fail(#fmt["Expected an int_value but found %?", self]);
			}
		}
	}
	
	fn as_i64() -> i64
	{
		alt self
		{
			int_value(value)
			{
				value
			}
			_
			{
				fail(#fmt["Expected an int_value but found %?", self]);
			}
		}
	}
	
	fn as_float() -> float
	{
		alt self
		{
			int_value(value)
			{
				value as float
			}
			float_value(value)
			{
				value as float
			}
			_
			{
				fail(#fmt["Expected int_value or float_value but found %?", self]);
			}
		}
	}
	
	fn as_f64() -> f64
	{
		alt self
		{
			int_value(value)
			{
				value as f64
			}
			float_value(value)
			{
				value
			}
			_
			{
				fail(#fmt["Expected int_value or float_value but found %?", self]);
			}
		}
	}
	
	fn as_tm() -> tm
	{
		alt self
		{
			dateTime_value(value)
			{
				value
			}
			_
			{
				fail(#fmt["Expected a dateTime_value but found %?", self]);
			}
		}
	}
	
	fn as_str() -> str
	{
		alt self
		{
			string_value(value, _lang)
			{
				value
			}
			_
			{
				fail(#fmt["Expected a string_value but found %?", self]);
			}
		}
	}
	
	fn as_iri() -> str
	{
		alt self
		{
			iri_value(value)
			{
				value
			}
			_
			{
				fail(#fmt["Expected an iri_value but found %?", self]);
			}
		}
	}
	
	fn as_bool_or_default(default:bool) -> bool
	{
		alt self
		{
			bool_value(value)
			{
				value
			}
			_
			{
				default
			}
		}
	}
	
	fn as_int_or_default(default: int) -> int
	{
		alt self
		{
			int_value(value)
			{
				if value >= int::min_value as i64 && value <= int::max_value as i64
				{
					value as int
				}
				else
				{
					fail(#fmt["Can't convert %? to an int", self]);
				}
			}
			_
			{
				default
			}
		}
	}
	
	fn as_uint_or_default(default: uint) -> uint
	{
		alt self
		{
			int_value(value)
			{
				if value >= uint::min_value as i64 && value <= uint::max_value as i64
				{
					value as uint
				}
				else
				{
					fail(#fmt["Can't convert %? to a uint", self]);
				}
			}
			_
			{
				default
			}
		}
	}
	
	fn as_i64_or_default(default: i64) -> i64
	{
		alt self
		{
			int_value(value)
			{
				value
			}
			_
			{
				default
			}
		}
	}
	
	fn as_float_or_default(default:float) -> float
	{
		alt self
		{
			int_value(value)
			{
				value as float
			}
			float_value(value)
			{
				value as float
			}
			_
			{
				default
			}
		}
	}
	
	fn as_f64_or_default(default: f64) -> f64
	{
		alt self
		{
			int_value(value)
			{
				value as f64
			}
			float_value(value)
			{
				value
			}
			_
			{
				default
			}
		}
	}
	
	fn as_tm_or_default(default: tm) -> tm
	{
		alt self
		{
			dateTime_value(value)
			{
				value
			}
			_
			{
				default
			}
		}
	}
	
	fn as_str_or_default(default: str) -> str
	{
		alt self
		{
			string_value(value, _lang)
			{
				value
			}
			_
			{
				default
			}
		}
	}
	
	fn as_iri_or_default(default: str) -> str
	{
		alt self
		{
			iri_value(value)
			{
				value
			}
			_
			{
				default
			}
		}
	}
}

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			bool_value(value)
			{
				if value {"true"} else {"false"}
			}
			int_value(value)
			{
				#fmt["%?", value]
			}
			float_value(value)
			{
				#fmt["%?", value]
			}
			dateTime_value(value)
			{
				value.rfc3339()
			}
			string_value(value, lang)
			{
				if str::is_not_empty(lang) {#fmt["\"%s\"@%s", value, lang]} else {#fmt["\"%s\"", value]}
			}
			typed_value(value, kind)
			{
				#fmt["\"%s^^\"%s", value, kind]
			}
			iri_value(value)
			{
				"<" + value + ">"
			}
			blank_value(value)
			{
				value
			}
			unbound_value(name)
			{
				name + " is not bound"
			}
			invalid_value(literal, kind)
			{
				#fmt["'%s' is not a valid %s", literal, kind]
			}
			error_value(err)
			{
				err
			}
		}
	}
}

#[doc = "Converts an arbitrary lexical value to an object.

Note that it is usually simplest to simply use the object enum directly."]
fn literal_to_object(value: str, kind: str, lang: str) -> object
{
	alt (value, kind, lang)
	{
		(v, "blank", "")
		{
			blank_value(v)
		}
		(v, "http://www.w3.org/2001/XMLSchema#anyURI", "")
		{
			if str::starts_with(v, "_:")
			{
				blank_value(v)
			}
			else
			{
				iri_value(v)
			}
		}
		(v, "http://www.w3.org/2001/XMLSchema#boolean", "")
		{
			if v == "true" || v == "1"
			{
				bool_value(true)
			}
			else if v == "false" || v == "0"
			{
				bool_value(false)
			}
			else
			{
				invalid_value(v, kind)
			}
		}
		(v, "http://www.w3.org/2001/XMLSchema#dateTime", "")
		{
			// Time zone expressed as an offset from GMT, e.g. -05:00 for EST.
			alt std::time::strptime(v, "%FT%T%z").chain_err
				{|_err1|
					// Time zone expressed as a name, e.g. EST (technically only Z is supposed to be allowed).
					std::time::strptime(v, "%FT%T%Z").chain_err
					{|_err2|
						// No time zone (so the time will be considered to be in the local time zone).
						std::time::strptime(v, "%FT%T")
					}}
			{
				result::ok(time)
				{
					dateTime_value(time)
				}
				result::err(_)
				{
					// invalid_value would seem more sensible, but the standard explicitly
					// reserves that for bool and numeric.
					error_value(#fmt["'%s' is not an ISO 8601 dateTime.", v])
				}
			}
		}
		(v, "http://www.w3.org/2001/XMLSchema#decimal", "") |	// minimally conformant processors must support at least 18 digits and i64 gives us 19
		(v, "http://www.w3.org/2001/XMLSchema#integer", "") |
		(v, "http://www.w3.org/2001/XMLSchema#nonPositiveInteger", "") |
		(v, "http://www.w3.org/2001/XMLSchema#negativeInteger", "") |
		(v, "http://www.w3.org/2001/XMLSchema#long", "") |
		(v, "http://www.w3.org/2001/XMLSchema#int", "") |
		(v, "http://www.w3.org/2001/XMLSchema#short", "") |
		(v, "http://www.w3.org/2001/XMLSchema#byte", "") |
		(v, "http://www.w3.org/2001/XMLSchema#nonNegativeInteger", "") |
		(v, "http://www.w3.org/2001/XMLSchema#unsignedLong", "") |
		(v, "http://www.w3.org/2001/XMLSchema#unsignedInt", "") |
		(v, "http://www.w3.org/2001/XMLSchema#unsignedShort", "") |
		(v, "http://www.w3.org/2001/XMLSchema#unsignedByte", "") |
		(v, "http://www.w3.org/2001/XMLSchema#positiveInteger", "")
		{
			str::as_c_str(v)
			{|vp|
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(end);
				let r = libc::strtol(vp, ptr::addr_of(endp), 10 as libc::c_int);
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						int_value(r as i64)
					}
					else
					{
						invalid_value(v, kind)
					}
				}
			}
		}
		(v, "http://www.w3.org/2001/XMLSchema#float", "") | 
		(v, "http://www.w3.org/2001/XMLSchema#double", "")
		{
			str::as_c_str(v)
			{|vp|
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(end);
				let r = libc::strtod(vp, ptr::addr_of(endp));
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						float_value(r as f64)
					}
					else
					{
						invalid_value(v, kind)
					}
				}
			}
		}
		(v, "http://www.w3.org/2001/XMLSchema#string", l) |
		(v, "http://www.w3.org/2001/XMLSchema#normalizedString", l) |
		(v, "http://www.w3.org/2001/XMLSchema#token", l) |
		(v, "http://www.w3.org/2001/XMLSchema#language", l) |
		(v, "http://www.w3.org/2001/XMLSchema#Name", l) |
		(v, "http://www.w3.org/2001/XMLSchema#NCName", l) |
		(v, "http://www.w3.org/2001/XMLSchema#ID", l)
		{
			string_value(v, l)
		}
		(v, k, "")
		{
			typed_value(v, k)
		}
		_
		{
			#error["object_to_operand unsupported type: %s.", kind];
			error_value(#fmt["object_to_operand unsupported type: %s.", kind])
		}
	}
}

fn get_object(row: solution_row, name: str) -> object
{
	alt row.search(name)
	{
		option::some(value)
		{
			value
		}
		option::none
		{
			unbound_value(name)
		}
	}
}

// Effective boolean value, see 17.2.2
fn get_ebv(operand: object) -> result::result<bool, str>
{
	alt operand
	{
		invalid_value(_literal, _type)
		{
			result::ok(false)
		}
		bool_value(value)
		{
			result::ok(value)
		}
		string_value(value, _) | typed_value(value, _)
		{
			result::ok(str::is_not_empty(value))
		}
		int_value(value)
		{
			result::ok(value != 0i64)
		}
		float_value(value)
		{
			result::ok(!f64::is_NaN(value) && value != 0f64)
		}
		unbound_value(name)
		{
			result::err(#fmt["?%s is not bound.", name])
		}
		error_value(err)
		{
			result::err(err)
		}
		_
		{
			result::err(#fmt["%? cannot be converted into an effective boolean value.", operand])
		}
	}
}

fn type_error(fname: str, operand: object, expected: str) -> str
{
	alt operand
	{
		unbound_value(name)
		{
			#fmt["%s: ?%s was not bound.", fname, name]
		}
		invalid_value(literal, kind)
		{
			#fmt["%s: '%s' is not a valid %s", fname, literal, kind]
		}
		error_value(err)
		{
			#fmt["%s: %s", fname, err]
		}
		_
		{
			#fmt["%s: expected %s value but found %?.", fname, expected, operand]
		}
	}
}

