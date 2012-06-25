// Operators used within SPARQL FILTER expressions. See 17.2 and related.
import operands::*;

export op_not, op_unary_plus, op_unary_minus, op_or, op_and, op_equals, op_not_equals,
	op_less_than, op_less_than_or_equal, op_greater_than, op_greater_than_or_equal;

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

fn equal_values(operator: str, lhs: operand, rhs: operand) -> result::result<bool, str>
{
	alt lhs
	{
		bool_value(lvalue)
		{
			alt rhs
			{
				bool_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "boolean"))
				}
			}
		}
		int_value(lvalue)
		{
			alt rhs
			{
				int_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				float_value(rvalue)
				{
					result::ok(lvalue as f64 == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "numeric"))
				}
			}
		}
		float_value(lvalue)
		{
			alt rhs
			{
				int_value(rvalue)
				{
					result::ok(lvalue == rvalue as f64)
				}
				float_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "numeric"))
				}
			}
		}
		dateTime_value(lvalue)
		{
			alt rhs
			{
				dateTime_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "dateTime"))
				}
			}
		}
		string_value(lvalue, llang)
		{
			alt rhs
			{
				string_value(rvalue, rlang)
				{
					result::ok(str::to_lower(llang) == str::to_lower(rlang) && lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "string"))
				}
			}
		}
		typed_value(lvalue, ltype)
		{
			alt rhs
			{
				typed_value(rvalue, rtype)
				{
					result::ok(ltype == rtype && lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, ltype))
				}
			}
		}
		iri_value(lvalue)
		{
			alt rhs
			{
				iri_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "IRI"))
				}
			}
		}
		blank_value(lvalue)
		{
			alt rhs
			{
				blank_value(rvalue)
				{
					result::ok(lvalue == rvalue)
				}
				_
				{
					result::err(type_error(operator, rhs, "blank"))
				}
			}
		}
		_
		{
			result::err(type_error(operator, lhs, "a"))
		}
	}
}

fn compare_values(operator: str, lhs: operand, rhs: operand) -> result::result<int, str>
{
	alt lhs
	{
		int_value(lvalue)
		{
			alt rhs
			{
				int_value(rvalue)
				{
					result::ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				float_value(rvalue)
				{
					let lvalue = lvalue as f64;
					result::ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				_
				{
					result::err(type_error(operator, rhs, "numeric"))
				}
			}
		}
		float_value(lvalue)
		{
			alt rhs
			{
				int_value(rvalue)
				{
					let rvalue = rvalue as f64;
					result::ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				float_value(rvalue)
				{
					result::ok(if lvalue < rvalue {-1} else if lvalue == rvalue {0} else {1})
				}
				_
				{
					result::err(type_error(operator, rhs, "numeric"))
				}
			}
		}
		dateTime_value(lvalue)
		{
			alt rhs
			{
				dateTime_value(rvalue)
				{
					let lvalue = lvalue.to_timespec();
					let rvalue = rvalue.to_timespec();
					result::ok(
						if lvalue.sec < rvalue.sec || (lvalue.sec == rvalue.sec && lvalue.nsec < rvalue.nsec) {-1} 
						else if lvalue.sec == rvalue.sec && lvalue.nsec == rvalue.nsec {0} 
						else {1}
					)
				}
				_
				{
					result::err(type_error(operator, rhs, "dateTime"))
				}
			}
		}
		string_value(lvalue, llang)
		{
			alt rhs
			{
				string_value(rvalue, rlang)
				{
					let llang = str::to_lower(llang);
					let rlang = str::to_lower(rlang);
					result::ok(
						if llang < rlang || (llang == rlang && lvalue < rvalue) {-1} 
						else if llang == rlang && lvalue == rvalue {0} 
						else {1}
					)
				}
				_
				{
					result::err(type_error(operator, rhs, "string"))
				}
			}
		}
		typed_value(lvalue, ltype)
		{
			alt rhs
			{
				typed_value(rvalue, rtype)
				{
					result::ok(
						if ltype < rtype || (ltype == rtype && lvalue < rvalue) {-1} 
						else if ltype == rtype && lvalue == rvalue {0} 
						else {1}
					)
				}
				_
				{
					result::err(type_error(operator, rhs, ltype))
				}
			}
		}
		_
		{
			result::err(type_error(operator, lhs, "numeric, dateTime, string, or explicitly typed"))
		}
	}
}

// ---- Unary Operators -------------------------------------------------------
fn op_not(operand: operand) -> operand
{
	alt get_ebv(operand)
	{
		result::ok(value)
		{
			bool_value(!value)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_unary_plus(operand: operand) -> operand
{
	alt operand
	{
		int_value(value)
		{
			operand
		}
		float_value(value)
		{
			operand
		}
		_
		{
			error_value(type_error("unary plus", operand, "numeric"))
		}
	}
}

fn op_unary_minus(operand: operand) -> operand
{
	alt operand
	{
		int_value(value)
		{
			int_value(-value)
		}
		float_value(value)
		{
			float_value(-value)
		}
		_
		{
			error_value(type_error("unary minus", operand, "numeric"))
		}
	}
}

// ---- Binary Operators -------------------------------------------------------
fn op_or(lhs: operand, rhs: operand) -> operand
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_success(lvalue) && result::is_success(rvalue)
	{
		bool_value(result::get(lvalue) || result::get(rvalue))
	}
	else if result::is_success(lvalue)
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
	else if result::is_success(rvalue)
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
		error_value(#fmt["%s %s", result::get_err(lvalue), result::get_err(rvalue)])
	}
}

fn op_and(lhs: operand, rhs: operand) -> operand
{
	let lvalue = get_ebv(lhs);
	let rvalue = get_ebv(rhs);
	
	if result::is_success(lvalue) && result::is_success(rvalue)
	{
		bool_value(result::get(lvalue) && result::get(rvalue))
	}
	else if result::is_success(lvalue)
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
	else if result::is_success(rvalue)
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
		error_value(#fmt["%s %s", result::get_err(lvalue), result::get_err(rvalue)])
	}
}

fn op_equals(lhs: operand, rhs: operand) -> operand
{
	alt equal_values("=", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(value)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_not_equals(lhs: operand, rhs: operand) -> operand
{
	alt equal_values("!=", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(!value)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_less_than(lhs: operand, rhs: operand) -> operand
{
	alt compare_values("<", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(value < 0)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_less_than_or_equal(lhs: operand, rhs: operand) -> operand
{
	alt compare_values("<=", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(value <= 0)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_greater_than(lhs: operand, rhs: operand) -> operand
{
	alt compare_values(">", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(value > 0)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

fn op_greater_than_or_equal(lhs: operand, rhs: operand) -> operand
{
	alt compare_values(">=", lhs, rhs)
	{
		result::ok(value)
		{
			bool_value(value >= 0)
		}
		result::err(err)
		{
			error_value(err)
		}
	}
}

// TODO:
// multiply
// divide
// add
// subtract
