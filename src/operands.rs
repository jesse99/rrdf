// Values used within SPARQL FILTER expressions. See 17.2 and related.
import result::extensions;

// Note that the error conditions do not always result in an error:
// 1) Functions like COALESCE accept unbound variables.
// 2) Boolean functions normally want effective boolean values which are false for invalid values.
// 3) Functions like op_and do not always propagate errors.
enum operand
{
	// literals
	bool_value(bool),
	int_value(i64),			// xsd:decimal (and derived types)
	float_value(f64),			// xsd:float or xsd:double
	dateTime_value(tm),	// xsd:dateTime
	string_value(str, str),	// value + lang
	typed_value(str, str),	// value + type (aka simple literal)
	
	// other rdf terms
	iri_value(str),
	blank_value(str),
	
	// error conditions
	unbound_value(str),	// binding name
	invalid_value(str),		// err mesg (for literal with invalid representation)
	error_value(str)			// err mesg
}

fn object_to_operand(value: object) -> operand
{
	alt value
	{
		{value: v, kind: "blank", lang: ""}
		{
			blank_value(v)
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}
		{
			iri_value(v)
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#boolean", lang: ""}
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
				invalid_value(#fmt["'%s' is not a valid boolean", v])
			}
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#dateTime", lang: ""}
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
					error_value(#fmt["'%s' is not an ISO 8601 dateTime", v])
				}
			}
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#decimal", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#nonPositiveInteger", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#negativeInteger", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#long", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#int", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#short", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#byte", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#unsignedLong", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#unsignedInt", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#unsignedShort", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#unsignedByte", lang: ""} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#positiveInteger", lang: ""}
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
						invalid_value(#fmt["'%s' is not a valid integer.", v])
					}
				}
			}
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#float", lang: ""} | 
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#double", lang: ""}
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
						invalid_value(#fmt["'%s' is not a valid floating point number.", v])
					}
				}
			}
		}
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#string", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#normalizedString", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#token", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#language", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#Name", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#NCName", lang: l} |
		{value: v, kind: "http://www.w3.org/2001/XMLSchema#ID", lang: l}
		{
			string_value(v, l)
		}
		{value: v, kind: k, lang: ""}
		{
			typed_value(v, k)
		}
		_
		{
			#error["object_to_operand unsupported type: %s.", value.kind];
			error_value(#fmt["object_to_operand unsupported type: %s.", value.kind])
		}
	}
}

fn get_operand(row: solution_row, name: str) -> operand
{
	alt row.search(name)
	{
		option::some(value)
		{
			object_to_operand(value)
		}
		option::none
		{
			unbound_value(name)
		}
	}
}

// 17.2.2
fn get_ebv(operand: operand) -> result::result<bool, str>
{
	alt operand
	{
		invalid_value(_err)
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

fn type_error(fname: str, operand: operand, expected: str) -> str
{
	alt operand
	{
		unbound_value(name)
		{
			#fmt["%s: ?%s was not bound.", fname, name]
		}
		invalid_value(err)
		{
			#fmt["%s: %s", fname, err]
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

