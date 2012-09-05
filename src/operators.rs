//! SPARQL operators. Clients will not ordinarily use this.
use object::*;

// Operators used within SPARQL FILTER expressions. See 17.2 and related.
export op_not, op_unary_plus, op_unary_minus, op_or, op_and, op_equals, op_not_equals,
	op_less_than, op_less_than_or_equal, op_greater_than, op_greater_than_or_equal,
	op_multiply, op_divide, op_add, op_subtract, compare_values;
	
fn equal_values(operator: ~str, lhs: object, rhs: object) -> result::Result<bool, ~str>
{
	match lhs
	{
		bool_value(lvalue) =>
		{
			match rhs
			{
				bool_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"boolean"))
				}
			}
		}
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				float_value(rvalue) =>
				{
					result::Ok(lvalue as f64 == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue as f64)
				}
				float_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		dateTime_value(lvalue) =>
		{
			match rhs
			{
				dateTime_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"dateTime"))
				}
			}
		}
		string_value(lvalue, llang) =>
		{
			match rhs
			{
				string_value(rvalue, rlang) =>
				{
					result::Ok(str::to_lower(llang) == str::to_lower(rlang) && lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"string"))
				}
			}
		}
		typed_value(lvalue, ltype) =>
		{
			match rhs
			{
				typed_value(rvalue, rtype) =>
				{
					result::Ok(ltype == rtype && lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ltype))
				}
			}
		}
		iri_value(lvalue) =>
		{
			match rhs
			{
				iri_value(rvalue) =>
				{
					result::Ok(lvalue == rvalue)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"IRI"))
				}
			}
		}
		blank_value(lvalue) =>
		{
			match rhs
			{
				blank_value(rvalue) =>
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
fn compare_values(operator: ~str, lhs: object, rhs: object) -> result::Result<int, ~str>
{
	match lhs
	{
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				float_value(rvalue) =>
				{
					let lvalue = lvalue as f64;
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					let rvalue = rvalue as f64;
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				float_value(rvalue) =>
				{
					result::Ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"numeric"))
				}
			}
		}
		dateTime_value(lvalue) =>
		{
			match rhs
			{
				dateTime_value(rvalue) =>
				{
					let lvalue = lvalue.to_timespec();
					let rvalue = rvalue.to_timespec();
					result::Ok(
						if lvalue.sec < rvalue.sec || (lvalue.sec == rvalue.sec && lvalue.nsec < rvalue.nsec) {-1} 
						else if lvalue.sec == rvalue.sec && lvalue.nsec == rvalue.nsec {0} 
						else {1}
					)
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"dateTime"))
				}
			}
		}
		string_value(lvalue, llang) =>
		{
			match rhs
			{
				string_value(rvalue, rlang) =>
				{
					let llang = str::to_lower(llang);
					let rlang = str::to_lower(rlang);
					result::Ok(
						if llang < rlang || (llang == rlang && lvalue < rvalue) {-1} 
						else if llang == rlang && lvalue == rvalue {0} 
						else {1}
					)
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"string"))
				}
			}
		}
		typed_value(lvalue, ltype) =>
		{
			match rhs
			{
				typed_value(rvalue, rtype) =>
				{
					result::Ok(
						if ltype < rtype || (ltype == rtype && lvalue < rvalue) {-1} 
						else if ltype == rtype && lvalue == rvalue {0} 
						else {1}
					)
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ltype))
				}
			}
		}
		iri_value(lvalue) =>
		{
			match rhs
			{
				iri_value(rvalue) =>
				{
					result::Ok(
						if lvalue < rvalue {-1} 
						else if lvalue == rvalue {0} 
						else {1}
					)
				}
				unbound_value(_) | blank_value(_) =>
				{
					result::Ok(1)
				}
				_ =>
				{
					result::Err(type_error(operator, rhs, ~"anyURI"))
				}
			}
		}
		unbound_value(_) =>
		{
			match rhs
			{
				unbound_value(_) =>
				{
					result::Ok(0)
				}
				_ =>
				{
					result::Ok(-1)
				}
			}
		}
		blank_value(lvalue) =>
		{
			match rhs
			{
				unbound_value(_) =>
				{
					result::Ok(1)
				}
				blank_value(rvalue) =>
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
fn op_not(operand: object) -> object
{
	match get_ebv(operand)
	{
		result::Ok(value) =>
		{
			bool_value(!value)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_unary_plus(operand: object) -> object
{
	match operand
	{
		int_value(_) =>
		{
			operand
		}
		float_value(_) =>
		{
			operand
		}
		_ =>
		{
			error_value(type_error(~"unary plus", operand, ~"numeric"))
		}
	}
}

fn op_unary_minus(operand: object) -> object
{
	match operand
	{
		int_value(value) =>
		{
			int_value(-value)
		}
		float_value(value) =>
		{
			float_value(-value)
		}
		_ =>
		{
			error_value(type_error(~"unary minus", operand, ~"numeric"))
		}
	}
}

// ---- Binary Operators -------------------------------------------------------
fn op_or(lhs: object, rhs: object) -> object
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_ok(lvalue) && result::is_ok(rvalue)
	{
		bool_value(result::get(lvalue) || result::get(rvalue))
	}
	else if result::is_ok(lvalue)
	{
		if result::get(lvalue)
		{
			bool_value(true)
		}
		else
		{
			error_value(result::get_err(rvalue))
		}
	}
	else if result::is_ok(rvalue)
	{
		if result::get(rvalue)
		{
			bool_value(true)
		}
		else
		{
			error_value(result::get_err(lvalue))
		}
	}
	else
	{
		error_value(fmt!("%s %s", result::get_err(lvalue), result::get_err(rvalue)))
	}
}

fn op_and(lhs: object, rhs: object) -> object
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_ok(lvalue) && result::is_ok(rvalue)
	{
		bool_value(result::get(lvalue) && result::get(rvalue))
	}
	else if result::is_ok(lvalue)
	{
		if !result::get(lvalue)
		{
			bool_value(false)
		}
		else
		{
			error_value(result::get_err(rvalue))
		}
	}
	else if result::is_ok(rvalue)
	{
		if !result::get(rvalue)
		{
			bool_value(false)
		}
		else
		{
			error_value(result::get_err(lvalue))
		}
	}
	else
	{
		error_value(fmt!("%s %s", result::get_err(lvalue), result::get_err(rvalue)))
	}
}

fn op_equals(lhs: object, rhs: object) -> object
{
	match equal_values(~"=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(value)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_not_equals(lhs: object, rhs: object) -> object
{
	match equal_values(~"!=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(!value)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_less_than(lhs: object, rhs: object) -> object
{
	match compare_values(~"<", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(value < 0)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_less_than_or_equal(lhs: object, rhs: object) -> object
{
	match compare_values(~"<=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(value <= 0)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_greater_than(lhs: object, rhs: object) -> object
{
	match compare_values(~">", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(value > 0)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_greater_than_or_equal(lhs: object, rhs: object) -> object
{
	match compare_values(~">=", lhs, rhs)
	{
		result::Ok(value) =>
		{
			bool_value(value >= 0)
		}
		result::Err(err) =>
		{
			error_value(err)
		}
	}
}

fn op_multiply(lhs: object, rhs: object) -> object
{
	match lhs
	{
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					int_value(lvalue*rvalue)
				}
				float_value(rvalue) =>
				{
					let lvalue = lvalue as f64;
					float_value(lvalue*rvalue)
				}
				_ =>
				{
					error_value(type_error(~"*", rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					let rvalue = rvalue as f64;
					float_value(lvalue*rvalue)
				}
				float_value(rvalue) =>
				{
					float_value(lvalue*rvalue)
				}
				_ =>
				{
					error_value(type_error(~"*", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			error_value(type_error(~"*", lhs, ~"numeric"))
		}
	}
}

fn op_divide(lhs: object, rhs: object) -> object
{
	match lhs
	{
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(0i64) =>
				{
					error_value(~"Divide by zero.")
				}
				int_value(rvalue) =>
				{
					int_value(lvalue/rvalue)
				}
				float_value(rvalue) =>
				{
					let lvalue = lvalue as f64;
					float_value(lvalue/rvalue)
				}
				_ =>
				{
					error_value(type_error(~"/", rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					let rvalue = rvalue as f64;
					float_value(lvalue/rvalue)
				}
				float_value(rvalue) =>
				{
					float_value(lvalue/rvalue)
				}
				_ =>
				{
					error_value(type_error(~"/", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			error_value(type_error(~"/", lhs, ~"numeric"))
		}
	}
}

fn op_add(lhs: object, rhs: object) -> object
{
	match lhs
	{
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					int_value(lvalue+rvalue)
				}
				float_value(rvalue) =>
				{
					let lvalue = lvalue as f64;
					float_value(lvalue+rvalue)
				}
				_ =>
				{
					error_value(type_error(~"+", rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					let rvalue = rvalue as f64;
					float_value(lvalue+rvalue)
				}
				float_value(rvalue) =>
				{
					float_value(lvalue+rvalue)
				}
				_ =>
				{
					error_value(type_error(~"+", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			error_value(type_error(~"+", lhs, ~"numeric"))
		}
	}
}

fn op_subtract(lhs: object, rhs: object) -> object
{
	match lhs
	{
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					int_value(lvalue-rvalue)
				}
				float_value(rvalue) =>
				{
					let lvalue = lvalue as f64;
					float_value(lvalue-rvalue)
				}
				_ =>
				{
					error_value(type_error(~"-", rhs, ~"numeric"))
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					let rvalue = rvalue as f64;
					float_value(lvalue-rvalue)
				}
				float_value(rvalue) =>
				{
					float_value(lvalue-rvalue)
				}
				_ =>
				{
					error_value(type_error(~"-", rhs, ~"numeric"))
				}
			}
		}
		_ =>
		{
			error_value(type_error(~"-", lhs, ~"numeric"))
		}
	}
}
