//! Value component of a triple and associated methods.
use std::time::{Tm};

// Object must be sendable (so that solutions can be sent cross-task).

// Note that the SPARQL error conditions do not always result in an error:
// 1) Functions like COALESCE accept unbound variables.
// 2) Boolean functions normally want effective boolean values which are false for invalid values.
// 3) Functions like op_and do not always propagate errors.
/// Value component of a triple.
enum Object				// TODO: once we support serialization we'll need to add something like u8 type codes to int, float, and string values
{								// TODO: predicate could maybe be enum with type code and uri
	// literals
	BoolValue(bool),
	IntValue(i64),				// value, xsd:decimal (and derived types)
	FloatValue(f64),			// value, xsd:float or xsd:double
	DateTimeValue(Tm),		// xsd:dateTime
	StringValue(~str, ~str),	// value + lang
	TypedValue(~str, ~str),	// value + type iri (aka simple literal)
	
	// other rdf terms
	IriValue(~str),
	BlankValue(~str),
	
	// error conditions
	UnboundValue(~str),		// binding name
	InvalidValue(~str, ~str),	// literal + type iri
	ErrorValue(~str)			// err mesg
}

impl Object
{
	fn to_friendly_str(namespaces: &[{prefix: ~str, path: ~str}]) -> ~str
	{
		match self
		{
			TypedValue(value, kind) =>
			{
				fmt!("\"%s^^\"%s", value, store::contract_uri(namespaces, kind))
			}
			IriValue(iri) =>
			{
				let result = store::contract_uri(namespaces, iri);
				if result != iri
				{
					result
				}
				else
				{
					~"<" + iri + ~">"
				}
			}
			_ =>
			{
				self.to_str()
			}
		}
	}
	
	fn as_bool() -> bool
	{
		match self
		{
			BoolValue(value) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected a BoolValue but found %?", self));
			}
		}
	}
	
	fn as_int() -> int
	{
		match self
		{
			IntValue(value) =>
			{
				if value >= int::min_value as i64 && value <= int::max_value as i64
				{
					value as int
				}
				else
				{
					fail(fmt!("Can't convert %? to an int", self));
				}
			}
			_ =>
			{
				fail(fmt!("Expected an IntValue but found %?", self));
			}
		}
	}
	
	fn as_uint() -> uint
	{
		match self
		{
			IntValue(value) =>
			{
				if value >= uint::min_value as i64 && value <= uint::max_value as i64
				{
					value as uint
				}
				else
				{
					fail(fmt!("Can't convert %? to a uint", self));
				}
			}
			_ =>
			{
				fail(fmt!("Expected an IntValue but found %?", self));
			}
		}
	}
	
	fn as_i64() -> i64
	{
		match self
		{
			IntValue(value) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected an IntValue but found %?", self));
			}
		}
	}
	
	fn as_float() -> float
	{
		match self
		{
			IntValue(value) =>
			{
				value as float
			}
			FloatValue(value) =>
			{
				value as float
			}
			_ =>
			{
				fail(fmt!("Expected IntValue or FloatValue but found %?", self));
			}
		}
	}
	
	fn as_f64() -> f64
	{
		match self
		{
			IntValue(value) =>
			{
				value as f64
			}
			FloatValue(value) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected IntValue or FloatValue but found %?", self));
			}
		}
	}
	
	fn as_tm() -> Tm
	{
		match self
		{
			DateTimeValue(value) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected a DateTimeValue but found %?", self));
			}
		}
	}
	
	fn as_str() -> ~str
	{
		match self
		{
			StringValue(value, _lang) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected a StringValue but found %?", self));
			}
		}
	}
	
	fn as_iri() -> ~str
	{
		match self
		{
			IriValue(value) =>
			{
				value
			}
			_ =>
			{
				fail(fmt!("Expected an IriValue but found %?", self));
			}
		}
	}
	
	fn as_bool_or_default(default:bool) -> bool
	{
		match self
		{
			BoolValue(value) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_int_or_default(default: int) -> int
	{
		match self
		{
			IntValue(value) =>
			{
				if value >= int::min_value as i64 && value <= int::max_value as i64
				{
					value as int
				}
				else
				{
					fail(fmt!("Can't convert %? to an int", self));
				}
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_uint_or_default(default: uint) -> uint
	{
		match self
		{
			IntValue(value) =>
			{
				if value >= uint::min_value as i64 && value <= uint::max_value as i64
				{
					value as uint
				}
				else
				{
					fail(fmt!("Can't convert %? to a uint", self));
				}
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_i64_or_default(default: i64) -> i64
	{
		match self
		{
			IntValue(value) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_float_or_default(default:float) -> float
	{
		match self
		{
			IntValue(value) =>
			{
				value as float
			}
			FloatValue(value) =>
			{
				value as float
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_f64_or_default(default: f64) -> f64
	{
		match self
		{
			IntValue(value) =>
			{
				value as f64
			}
			FloatValue(value) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_tm_or_default(default: Tm) -> Tm
	{
		match self
		{
			DateTimeValue(value) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_str_or_default(default: ~str) -> ~str
	{
		match self
		{
			StringValue(value, _lang) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
	
	fn as_iri_or_default(default: ~str) -> ~str
	{
		match self
		{
			IriValue(value) =>
			{
				value
			}
			_ =>
			{
				default
			}
		}
	}
}

impl  Object : ToStr 
{
	fn to_str() -> ~str
	{
		match self
		{
			BoolValue(value) =>
			{
				if value {~"true"} else {~"false"}
			}
			IntValue(value) =>
			{
				fmt!("%?", value)
			}
			FloatValue(value) =>
			{
				fmt!("%?", value)
			}
			DateTimeValue(value) =>
			{
				value.rfc3339()
			}
			StringValue(value, lang) =>
			{
				if str::is_not_empty(lang) {fmt!("\"%s\"@%s", value, lang)} else {fmt!("\"%s\"", value)}
			}
			TypedValue(value, kind) =>
			{
				fmt!("\"%s^^\"%s", value, kind)
			}
			IriValue(value) =>
			{
				~"<" + value + ~">"
			}
			BlankValue(value) =>
			{
				value
			}
			UnboundValue(name) =>
			{
				name + ~" is not bound"
			}
			InvalidValue(literal, kind) =>
			{
				fmt!("'%s' is not a valid %s", literal, kind)
			}
			ErrorValue(err) =>
			{
				err
			}
		}
	}
}

/// Converts an arbitrary lexical value to an object.
/// 
/// Note that it is usually simplest to simply use the object enum directly.
fn literal_to_object(value: @~str, kind: @~str, lang: @~str) -> Object
{
	match (value, kind, lang)
	{
		(v, @~"blank", @~"") =>
		{
			BlankValue(*v)
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#anyURI", @~"") =>
		{
			if str::starts_with(*v, "_:")
			{
				BlankValue(*v)
			}
			else
			{
				IriValue(*v)
			}
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#boolean", @~"") =>
		{
			if v == @~"true" || v == @~"1"
			{
				BoolValue(true)
			}
			else if v == @~"false" || v == @~"0"
			{
				BoolValue(false)
			}
			else
			{
				InvalidValue(*v, *kind)
			}
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#dateTime", @~"") =>
		{
			// Time zone expressed as an offset from GMT, e.g. -05:00 for EST.
			match do std::time::strptime(*v, ~"%FT%T%z").chain_err
				|_err1|
				{
					// Time zone expressed as a name, e.g. EST (technically only Z is supposed to be allowed).
					do std::time::strptime(*v, ~"%FT%T%Z").chain_err
					|_err2|
					{
						// No time zone (so the time will be considered to be in the local time zone).
						std::time::strptime(*v, ~"%FT%T")
					}
				}
			{
				result::Ok(time) =>
				{
					DateTimeValue(time)
				}
				result::Err(_) =>
				{
					// InvalidValue would seem more sensible, but the standard explicitly
					// reserves that for bool and numeric.
					ErrorValue(fmt!("'%s' is not an ISO 8601 dateTime.", *v))
				}
			}
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#decimal", @~"") |	// minimally conformant processors must support at least 18 digits and i64 gives us 19
		(v, @~"http://www.w3.org/2001/XMLSchema#integer", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#nonPositiveInteger", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#negativeInteger", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#long", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#int", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#short", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#byte", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#nonNegativeInteger", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#unsignedLong", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#unsignedInt", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#unsignedShort", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#unsignedByte", @~"") |
		(v, @~"http://www.w3.org/2001/XMLSchema#positiveInteger", @~"") =>
		{
			do str::as_c_str(*v)
			|vp|
			{
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(end);
				let r = libc::strtol(vp, ptr::addr_of(endp), 10 as libc::c_int);
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						IntValue(r as i64)
					}
					else
					{
						InvalidValue(*v, *kind)
					}
				}
			}
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#float", @~"") | 
		(v, @~"http://www.w3.org/2001/XMLSchema#double", @~"") =>
		{
			do str::as_c_str(*v)
			|vp|
			{
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(end);
				let r = libc::strtod(vp, ptr::addr_of(endp));
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						FloatValue(r as f64)
					}
					else
					{
						InvalidValue(*v, *kind)
					}
				}
			}
		}
		(v, @~"http://www.w3.org/2001/XMLSchema#string", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#normalizedString", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#token", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#language", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#Name", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#NCName", l) |
		(v, @~"http://www.w3.org/2001/XMLSchema#ID", l) =>
		{
			StringValue(*v, *l)
		}
		(v, k, @~"") =>
		{
			TypedValue(*v, *k)
		}
		_ =>
		{
			error!("object_to_operand unsupported type: %s.", *kind);
			ErrorValue(fmt!("object_to_operand unsupported type: %s.", *kind))
		}
	}
}

// Effective boolean value, see 17.2.2
pure fn get_ebv(operand: Object) -> result::Result<bool, ~str>
{
	match operand
	{
		InvalidValue(_literal, _type) =>
		{
			result::Ok(false)
		}
		BoolValue(value) =>
		{
			result::Ok(value)
		}
		StringValue(value, _) | TypedValue(value, _) =>
		{
			result::Ok(str::is_not_empty(value))
		}
		IntValue(value) =>
		{
			result::Ok(value != 0i64)
		}
		FloatValue(value) =>
		{
			result::Ok(!f64::is_NaN(value) && value != 0f64)
		}
		UnboundValue(name) =>
		{
			result::Err(fmt!("?%s is not bound.", name))
		}
		ErrorValue(err) =>
		{
			result::Err(err)
		}
		_ =>
		{
			result::Err(fmt!("%? cannot be converted into an effective boolean value.", operand))
		}
	}
}

fn type_error(fname: ~str, operand: Object, expected: ~str) -> ~str
{
	match operand
	{
		UnboundValue(name) =>
		{
			fmt!("%s: ?%s was not bound.", fname, name)
		}
		InvalidValue(literal, kind) =>
		{
			fmt!("%s: '%s' is not a valid %s", fname, literal, kind)
		}
		ErrorValue(err) =>
		{
			fmt!("%s: %s", fname, err)
		}
		_ =>
		{
			fmt!("%s: expected %s value but found %?.", fname, expected, operand)
		}
	}
}

