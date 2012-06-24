import io;
import io::writer_util;
import operands::*;
import operators::*;

fn check_operands(actual: operand, expected: operand) -> bool
{
	if actual != expected
	{
		io::stderr().write_line(#fmt["Found %?, but expected %?", actual, expected]);
		ret false;
	}
	ret true;
}

// Also tests get_ebv.
#[test]
fn operator_not()
{
	assert check_operands(op_not(invalid_value("oops")), bool_value(true));

	assert check_operands(op_not(bool_value(false)), bool_value(true));
	
	assert check_operands(op_not(string_value("hello", "en")), bool_value(false));
	assert check_operands(op_not(typed_value("", "my-type")), bool_value(true));
	
	assert check_operands(op_not(int_value(23i64)), bool_value(false));
	assert check_operands(op_not(int_value(0i64)), bool_value(true));
	assert check_operands(op_not(float_value(2.3f64)), bool_value(false));
	assert check_operands(op_not(float_value(0.0f64)), bool_value(true));
	assert check_operands(op_not(float_value(f64::NaN)), bool_value(true));
}

#[test]
fn operator_unary_minus()
{
	assert check_operands(op_unary_minus(int_value(23i64)), int_value(-23i64));
	assert check_operands(op_unary_minus(float_value(2.3f64)), float_value(-2.3f64));
}

