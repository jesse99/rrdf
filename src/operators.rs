//! SPARQL operators. Clients will not ordinarily use this.
use object::*;

// Operators used within SPARQL FILTER expressions. See 17.2 and related.

pub fn equal_values(operator: ~str, lhs: &Object, rhs: &Object) -> result::Result<bool, ~str>
{
	match *lhs
	{
		BoolValue(lvalue) =>
		{
			match *rhs
			{
				BoolValue(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"boolean"))
				}
			}
		}
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				FloatValue(rvalue) =>
				{
					result::Ok(lvalue as f64 == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					result::Ok(lvalue == rvalue as f64)
				}
				FloatValue(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		DateTimeValue(ref lvalue) =>
		{
			match *rhs
			{
				DateTimeValue(ref rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"dateTime"))
				}
			}
		}
		StringValue(ref lvalue, ref llang) =>
		{
			match *rhs
			{
				StringValue(ref rvalue, ref rlang) =>
				{
					result::Ok(str::to_lower(*llang) == str::to_lower(*rlang) && lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"string"))
				}
			}
		}
		TypedValue(ref lvalue, ref ltype) =>
		{
			match *rhs
			{
				TypedValue(ref rvalue, ref rtype) =>
				{
					result::Ok(ltype == rtype && lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, copy *ltype))
				}
			}
		}
		IriValue(ref lvalue) =>
		{
			match *rhs
			{
				IriValue(ref rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"IRI"))
				}
			}
		}
		BlankValue(ref lvalue) =>
		{
			match *rhs
			{
				BlankValue(ref rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"blank"))
				}
			}
		}
		_ =>
		{
			result::Err(type_error(operator, lhs, ~"a"))
		}
	}
}

// See 15.1
pub fn compare_values(operator: ~str, lhs: &Object, rhs: &Object) -> result::Result<int, ~str>
{
	match *lhs
	{
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				FloatValue(rvalue) =>
				{
					let lvalue = lvalue as f64;
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					let rvalue = rvalue as f64;
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				FloatValue(rvalue) =>
				{
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		DateTimeValue(ref lvalue) =>
		{
			match *rhs
			{
				DateTimeValue(ref rvalue) =>
				{
					let lvalue = lvalue.to_timespec();
					let rvalue = rvalue.to_timespec();
					result::Ok(
						if lvalue.sec < rvalue.sec || (lvalue.sec == rvalue.sec && lvalue.nsec < rvalue.nsec) {-1} 
						else if lvalue.sec == rvalue.sec && lvalue.nsec == rvalue.nsec {0} 
						else {1}
					)
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"dateTime"))
				}
			}
		}
		StringValue(ref lvalue, ref llang) =>
		{
			match *rhs
			{
				StringValue(ref rvalue, ref rlang) =>
				{
					let llang = str::to_lower(*llang);
					let rlang = str::to_lower(*rlang);
					result::Ok(
						if llang < rlang || (llang == rlang && lvalue < rvalue) {-1} 
						else if llang == rlang && lvalue == rvalue {0} 
						else {1}
					)
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"string"))
				}
			}
		}
		TypedValue(ref lvalue, ref ltype) =>
		{
			match *rhs
			{
				TypedValue(ref rvalue, ref rtype) =>
				{
					result::Ok(
						if ltype < rtype || (ltype == rtype && lvalue < rvalue) {-1} 
						else if ltype == rtype && lvalue == rvalue {0} 
						else {1}
					)
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, copy *ltype))
				}
			}
		}
		IriValue(ref lvalue) =>
		{
			match *rhs
			{
				IriValue(ref rvalue) =>
				{
					result::Ok(
						if lvalue < rvalue {-1} 
						else if lvalue == rvalue {0} 
						else {1}
					)
				}
				UnboundValue(_) | BlankValue(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"anyURI"))
				}
			}
		}
		UnboundValue(_) =>
		{
			match *rhs
			{
				UnboundValue(_) =>
				{
					result::Ok(0)
				}
				_ =>
				{
					result::Ok(-1)
				}
			}
		}
		BlankValue(ref lvalue) =>
		{
			match *rhs
			{
				UnboundValue(_) =>
				{
					result::Ok(1)
				}
				BlankValue(ref rvalue) =>
				{
					result::Ok(
						if lvalue < rvalue {-1} 
						else if lvalue == rvalue {0} 
						else {1}
					)
				}
				_ =>
				{
					result::Ok(-1)
				}
			}
		}
		_ =>
		{
			result::Err(type_error(operator, lhs, ~"numeric, dateTime, string, or explicitly typed"))
		}
	}
}

// ---- Unary Operators -------------------------------------------------------
pub fn op_not(operand: &Object) -> Object
{
	match get_ebv(operand)
	{
		result::Ok(value) =>
		{
			BoolValue(!value)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_unary_plus(operand: &Object) -> Object
{
	match *operand
	{
		IntValue(_) =>
		{
			copy *operand
		}
		FloatValue(_) =>
		{
			copy *operand
		}
		_ =>
		{
			ErrorValue(type_error(~"unary plus", operand, ~"numeric"))
		}
	}
}

pub fn op_unary_minus(operand: &Object) -> Object
{
	match *operand
	{
		IntValue(value) =>
		{
			IntValue(-value)
		}
		FloatValue(value) =>
		{
			FloatValue(-value)
		}
		_ =>
		{
			ErrorValue(type_error(~"unary minus", operand, ~"numeric"))
		}
	}
}

// ---- Binary Operators -------------------------------------------------------
pub fn op_or(lhs: &Object, rhs: &Object) -> Object
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_ok(&lvalue) && result::is_ok(&rvalue)
	{
		BoolValue(result::get(&lvalue) || result::get(&rvalue))
	}
	else if result::is_ok(&lvalue)
	{
		if result::get(&lvalue)
		{
			BoolValue(true)
		}
		else
		{
			ErrorValue(result::get_err(&rvalue))
		}
	}
	else if result::is_ok(&rvalue)
	{
		if result::get(&rvalue)
		{
			BoolValue(true)
		}
		else
		{
			ErrorValue(result::get_err(&lvalue))
		}
	}
	else
	{
		ErrorValue(fmt!("%s %s", result::get_err(&lvalue), result::get_err(&rvalue)))
	}
}

pub fn op_and(lhs: &Object, rhs: &Object) -> Object
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_ok(&lvalue) && result::is_ok(&rvalue)
	{
		BoolValue(result::get(&lvalue) && result::get(&rvalue))
	}
	else if result::is_ok(&lvalue)
	{
		if !result::get(&lvalue)
		{
			BoolValue(false)
		}
		else
		{
			ErrorValue(result::get_err(&rvalue))
		}
	}
	else if result::is_ok(&rvalue)
	{
		if !result::get(&rvalue)
		{
			BoolValue(false)
		}
		else
		{
			ErrorValue(result::get_err(&lvalue))
		}
	}
	else
	{
		ErrorValue(fmt!("%s %s", result::get_err(&lvalue), result::get_err(&rvalue)))
	}
}

pub fn op_equals(lhs: &Object, rhs: &Object) -> Object
{
	match equal_values(~"=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(value)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_not_equals(lhs: &Object, rhs: &Object) -> Object
{
	match equal_values(~"!=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(!value)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_less_than(lhs: &Object, rhs: &Object) -> Object
{
	match compare_values(~"<", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(value < 0)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_less_than_or_equal(lhs: &Object, rhs: &Object) -> Object
{
	match compare_values(~"<=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(value <= 0)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_greater_than(lhs: &Object, rhs: &Object) -> Object
{
	match compare_values(~">", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(value > 0)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_greater_than_or_equal(lhs: &Object, rhs: &Object) -> Object
{
	match compare_values(~">=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			BoolValue(value >= 0)
		}
		result::Err(copy err) =>
		{
			ErrorValue(err)
		}
	}
}

pub fn op_multiply(lhs: &Object, rhs: &Object) -> Object
{
	match *lhs
	{
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					IntValue(lvalue*rvalue)
				}
				FloatValue(rvalue) =>
				{
					let lvalue = lvalue as f64;
					FloatValue(lvalue*rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"*", rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					let rvalue = rvalue as f64;
					FloatValue(lvalue*rvalue)
				}
				FloatValue(rvalue) =>
				{
					FloatValue(lvalue*rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"*", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			ErrorValue(type_error(~"*", lhs, ~"numeric"))
		}
	}
}

pub fn op_divide(lhs: &Object, rhs: &Object) -> Object
{
	match *lhs
	{
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(0i64) =>
				{
					ErrorValue(~"Divide by zero.")
				}
				IntValue(rvalue) =>
				{
					IntValue(lvalue/rvalue)
				}
				FloatValue(rvalue) =>
				{
					let lvalue = lvalue as f64;
					FloatValue(lvalue/rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"/", rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					let rvalue = rvalue as f64;
					FloatValue(lvalue/rvalue)
				}
				FloatValue(rvalue) =>
				{
					FloatValue(lvalue/rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"/", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			ErrorValue(type_error(~"/", lhs, ~"numeric"))
		}
	}
}

pub fn op_add(lhs: &Object, rhs: &Object) -> Object
{
	match *lhs
	{
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					IntValue(lvalue+rvalue)
				}
				FloatValue(rvalue) =>
				{
					let lvalue = lvalue as f64;
					FloatValue(lvalue+rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"+", rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					let rvalue = rvalue as f64;
					FloatValue(lvalue+rvalue)
				}
				FloatValue(rvalue) =>
				{
					FloatValue(lvalue+rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"+", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			ErrorValue(type_error(~"+", lhs, ~"numeric"))
		}
	}
}

pub fn op_subtract(lhs: &Object, rhs: &Object) -> Object
{
	match *lhs
	{
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					IntValue(lvalue-rvalue)
				}
				FloatValue(rvalue) =>
				{
					let lvalue = lvalue as f64;
					FloatValue(lvalue-rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"-", rhs, ~"numeric"))
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					let rvalue = rvalue as f64;
					FloatValue(lvalue-rvalue)
				}
				FloatValue(rvalue) =>
				{
					FloatValue(lvalue-rvalue)
				}
				_ =>
				{
					ErrorValue(type_error(~"-", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			ErrorValue(type_error(~"-", lhs, ~"numeric"))
		}
	}
}
