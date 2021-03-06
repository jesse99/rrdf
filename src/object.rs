//! Value component of a triple and associated methods.
use std::time::{Tm};

// Object must be sendable (so that solutions can be sent cross-task).

// Note that the SPARQL error conditions do not always result in an error:
// 1) Functions like COALESCE accept unbound variables.
// 2) Boolean functions normally want effective boolean values which are false for invalid values.
// 3) Functions like op_and do not always propagate errors.
/// Value component of a triple.
pub enum Object			// TODO: once we support serialization we'll need to add something like u8 type codes to int, float, and string values
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
	UnboundValue,
	InvalidValue(~str, ~str),	// literal + type iri
	ErrorValue(~str)			// err mesg
}

pub impl Object
{
	pure fn to_friendly_str(namespaces: &[Namespace]) -> ~str
	{
		match self
		{
			TypedValue(ref value, ref kind) =>
			{
				fmt!("\"%s^^\"%s", *value, contract_uri(namespaces, *kind))
			}
			IriValue(ref iri) =>
			{
				let result = contract_uri(namespaces, *iri);
				if result != *iri
				{
					result
				}
				else
				{
					~"<" + *iri + ~">"
				}
			}
			_ =>
			{
				self.to_str()
			}
		}
	}
	
	pure fn is_unbound() -> bool
	{
		match self
		{
			UnboundValue => true,
			_ => false,
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
			DateTimeValue(copy value) =>
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
			StringValue(copy value, ref _lang) =>
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
			IriValue(copy value) =>
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
			DateTimeValue(copy value) =>
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
			StringValue(copy value, ref _lang) =>
			{
				value
			}
			_ =>
			{
				copy default
			}
		}
	}
	
	fn as_iri_or_default(default: ~str) -> ~str
	{
		match self
		{
			IriValue(copy value) =>
			{
				value
			}
			_ =>
			{
				copy default
			}
		}
	}
}

priv pure fn get_key(o: &Object) -> uint
{
	match *o
	{
		BoolValue(*) => 1,
		IntValue(*) => 2,
		FloatValue(*) => 3,
		DateTimeValue(*) => 4,
		StringValue(*) => 5,
		TypedValue(*) => 6,
		IriValue(*) => 7,
		BlankValue(*) => 8,
		UnboundValue => 9,
		InvalidValue(*) => 10,
		ErrorValue(*) => 11,
	}
}

priv pure fn lt_(o1: &Object, o2: &Object) -> bool
{
	let k1 = get_key(o1);
	let k2 = get_key(o2);
	if k1 < k2
	{
		true
	}
	else if k1 > k2
	{
		false
	}
	else
	{
		match *o1
		{
			BoolValue(x1) => match *o2 {BoolValue(x2) => x1 < x2, _ => fail},
			IntValue(x1) => match *o2 {IntValue(x2) => x1 < x2, _ => fail},
			FloatValue(x1) => match *o2 {FloatValue(x2) => x1 < x2, _ => fail},
			DateTimeValue(ref x1) => match *o2
			{
				DateTimeValue(ref x2) =>
				{
					unsafe
					{
						let t1 = x1.to_timespec();
						let t2 = x2.to_timespec();
						t1.sec < t2.sec || (t1.sec == t2.sec && t1.nsec < t2.nsec)
					}
				}
				_ => fail,
			},
			StringValue(ref x1, ref y1) => match *o2 {StringValue(ref x2, ref y2) => x1 < x2 || (x1 == x2 && y1 < y2), _ => fail},
			TypedValue(ref x1, ref y1) => match *o2 {TypedValue(ref x2, ref y2) => x1 < x2 || (x1 == x2 && y1 < y2), _ => fail},
			IriValue(ref x1) => match *o2 {IriValue(ref x2) => x1 < x2, _ => fail},
			BlankValue(ref x1) => match *o2 {BlankValue(ref x2) => x1 < x2, _ => fail},
			UnboundValue => match *o2 {UnboundValue => false, _ => fail},
			InvalidValue(ref x1, ref y1) => match *o2 {InvalidValue(ref x2, ref y2) => x1 < x2 || (x1 == x2 && y1 < y2), _ => fail},
			ErrorValue(ref x1) => match *o2 {ErrorValue(ref x2) => x1 < x2, _ => fail},
		}
	}
}

pub impl Object : cmp::Ord
{
	pure fn lt(&self, other: &Object) -> bool
	{
		lt_(self, other)
	}
	
	pure fn le(&self, other: &Object) -> bool
	{
		!lt_(other, self)
	}
	
	pure fn gt(&self, other: &Object) -> bool
	{
		lt_(other, self)
	}
	
	pure fn ge(&self, other: &Object) -> bool
	{
		!lt_(self, other)
	}
}

pub impl Object : cmp::Eq
{
	pure fn eq(&self, other: &Object) -> bool
	{
		!lt_(self, other) && !lt_(other, self)
	}
	
	pure fn ne(&self, other: &Object) -> bool
	{
		lt_(self, other) || lt_(other, self)
	}
}

pub impl  Object : ToStr 
{
	pure fn to_str() -> ~str
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
			DateTimeValue(ref value) =>
			{
				unsafe {value.rfc3339()}
			}
			StringValue(ref value, ref lang) =>
			{
				if str::is_not_empty(*lang) {fmt!("\"%s\"@%s", *value, *lang)} else {fmt!("\"%s\"", *value)}
			}
			TypedValue(ref value, ref kind) =>
			{
				fmt!("\"%s^^\"%s", *value, *kind)
			}
			IriValue(ref value) =>
			{
				~"<" + *value + ~">"
			}
			BlankValue(copy value) =>
			{
				value
			}
			UnboundValue =>
			{
				~" unbound"
			}
			InvalidValue(ref literal, ref kind) =>
			{
				fmt!("'%s' is not a valid %s", *literal, *kind)
			}
			ErrorValue(copy err) =>
			{
				err
			}
		}
	}
}

/// Converts an arbitrary lexical value to an object.
/// 
/// Note that it is usually simplest to simply use the object enum directly.
pub fn literal_to_object(value: &str, kind: &str, lang: &str) -> Object
{
	match (value.to_owned(), kind.to_owned(), lang.to_owned())	// TODO: https://github.com/mozilla/rust/issues/3574
	{
		(copy v, ~"blank", ~"") =>
		{
			BlankValue(v)
		}
		(copy v, ~"http://www.w3.org/2001/XMLSchema#anyURI", ~"") =>
		{
			if str::starts_with(v, "_:")
			{
				BlankValue(v)
			}
			else
			{
				IriValue(v)
			}
		}
		(ref v, ~"http://www.w3.org/2001/XMLSchema#boolean", ~"") =>
		{
			if *v == ~"true" || *v == ~"1"
			{
				BoolValue(true)
			}
			else if *v == ~"false" || *v == ~"0"
			{
				BoolValue(false)
			}
			else
			{
				InvalidValue(copy *v, kind.to_owned())
			}
		}
		(ref v, ~"http://www.w3.org/2001/XMLSchema#dateTime", ~"") =>
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
				result::Ok(copy time) =>
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
		(ref v, ~"http://www.w3.org/2001/XMLSchema#decimal", ~"") |	// minimally conformant processors must support at least 18 digits and i64 gives us 19
		(ref v, ~"http://www.w3.org/2001/XMLSchema#integer", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#nonPositiveInteger", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#negativeInteger", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#long", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#int", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#short", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#byte", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#nonNegativeInteger", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#unsignedLong", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#unsignedInt", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#unsignedShort", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#unsignedByte", ~"") |
		(ref v, ~"http://www.w3.org/2001/XMLSchema#positiveInteger", ~"") =>
		{
			do str::as_c_str(*v)
			|vp|
			{
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(&end);
				let r = libc::strtol(vp, ptr::addr_of(&endp), 10 as libc::c_int);
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						IntValue(r as i64)
					}
					else
					{
						InvalidValue(v.to_owned(), kind.to_owned())
					}
				}
			}
		}
		(ref v, ~"http://www.w3.org/2001/XMLSchema#float", ~"") | 
		(ref v, ~"http://www.w3.org/2001/XMLSchema#double", ~"") =>
		{
			do str::as_c_str(*v)
			|vp|
			{
				let end = 0 as libc::c_char;
				let endp = ptr::addr_of(&end);
				let r = libc::strtod(vp, ptr::addr_of(&endp));
				unsafe
				{
					if *endp == 0 as libc::c_char
					{
						FloatValue(r as f64)
					}
					else
					{
						InvalidValue(v.to_owned(), kind.to_owned())
					}
				}
			}
		}
		(copy v, ~"http://www.w3.org/2001/XMLSchema#string", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#normalizedString", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#token", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#language", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#Name", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#NCName", copy l) |
		(copy v, ~"http://www.w3.org/2001/XMLSchema#ID", copy l) =>
		{
			StringValue(v, l)
		}
		(copy v, copy k, ~"") =>
		{
			TypedValue(v, k)
		}
		_ =>
		{
			error!("object_to_operand unsupported type: %s.", kind);
			ErrorValue(fmt!("object_to_operand unsupported type: %s.", kind))
		}
	}
}

// Effective boolean value, see 17.2.2
pub pure fn get_ebv(operand: &Object) -> result::Result<bool, ~str>
{
	match *operand
	{
		InvalidValue(*) =>
		{
			result::Ok(false)
		}
		BoolValue(value) =>
		{
			result::Ok(value)
		}
		StringValue(ref value, _) | TypedValue(ref value, _) =>
		{
			result::Ok(str::is_not_empty(*value))
		}
		IntValue(value) =>
		{
			result::Ok(value != 0i64)
		}
		FloatValue(value) =>
		{
			result::Ok(!f64::is_NaN(value) && value != 0f64)
		}
		UnboundValue =>
		{
			result::Err(~"unbound")
		}
		ErrorValue(copy err) =>
		{
			result::Err(err)
		}
		_ =>
		{
			result::Err(fmt!("%? cannot be converted into an effective boolean value.", operand))
		}
	}
}

pub pure fn type_error(fname: &str, operand: &Object, expected: ~str) -> ~str
{
	match *operand
	{
		UnboundValue =>
		{
			fmt!("%s: unbound.", fname)
		}
		InvalidValue(ref literal, ref kind) =>
		{
			fmt!("%s: '%s' is not a valid %s", fname, *literal, *kind)
		}
		ErrorValue(ref err) =>
		{
			fmt!("%s: %s", fname, *err)
		}
		_ =>
		{
			fmt!("%s: expected %s value but found %?.", fname, expected, *operand)
		}
	}
}

