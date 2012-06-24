// Operators used within SPARQL FILTER expressions. See 17.2 and related.
//import std::map::hashmap;
//import result::extensions;
//import std::time::tm;
//import sparql::*;
import operands::*;

//export join_solutions, eval, pattern;

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
			error_value(#fmt["Expected numeric value in unary plus but found %?", operand])
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
			error_value(#fmt["Expected numeric value in unary minus but found %?", operand])
		}
	}
}
