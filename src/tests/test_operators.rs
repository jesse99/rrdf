import io;
import io::writer_util;
import operands::*;
import operators::*;

fn check_operands(actual: operand, expected: operand) -> bool
{
	if actual != expected
	{
		io::stderr().write_line("Found:");
		io::stderr().write_line(#fmt["   %?", actual]);
		io::stderr().write_line("but expected:");
		io::stderr().write_line(#fmt["   %?", expected]);
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
	assert check_operands(op_unary_minus(string_value("oops", "")), error_value("unary minus: expected numeric value but found string_value(\"oops\", \"\")."));
}

#[test]
fn operator_or()
{
	assert check_operands(op_or(bool_value(true), bool_value(false)), bool_value(true));
	assert check_operands(op_or(bool_value(true), bool_value(false)), bool_value(true));
	
	assert check_operands(op_or(bool_value(true), error_value("oops.")), bool_value(true));
	assert check_operands(op_or(error_value("oops."), bool_value(true)), bool_value(true));
	
	assert check_operands(op_or(bool_value(false), error_value("oops.")), error_value("oops."));
	assert check_operands(op_or(error_value("oops."), bool_value(false)), error_value("oops."));
	
	assert check_operands(op_or(error_value("ack."), error_value("oops.")), error_value("ack. oops."));
}

#[test]
fn operator_and()
{
	assert check_operands(op_and(bool_value(true), bool_value(true)), bool_value(true));
	
	assert check_operands(op_and(bool_value(true), bool_value(false)), bool_value(false));
	assert check_operands(op_and(bool_value(true), bool_value(false)), bool_value(false));
	
	assert check_operands(op_and(bool_value(true), error_value("oops.")), error_value("oops."));
	assert check_operands(op_and(error_value("oops."), bool_value(true)), error_value("oops."));
	
	assert check_operands(op_and(bool_value(false), error_value("oops.")), bool_value(false));
	assert check_operands(op_and(error_value("oops."), bool_value(false)), bool_value(false));
	
	assert check_operands(op_and(error_value("ack."), error_value("oops.")), error_value("ack. oops."));
}

#[test]
fn operator_equals()
{
	assert check_operands(op_equals(bool_value(true), bool_value(true)), bool_value(true));
	assert check_operands(op_equals(bool_value(true), bool_value(false)), bool_value(false));
	
	assert check_operands(op_equals(int_value(3i64), int_value(3i64)), bool_value(true));
	assert check_operands(op_equals(int_value(3i64), int_value(4i64)), bool_value(false));
	assert check_operands(op_equals(int_value(3i64), float_value(4.0f64)), bool_value(false));
	assert check_operands(op_equals(int_value(3i64), float_value(3.0f64)), bool_value(true));
	assert check_operands(op_equals(float_value(3.0f64), float_value(3.0f64)), bool_value(true));
	
	assert check_operands(op_equals(string_value("foo", "en"), string_value("foo", "En")), bool_value(true));
	assert check_operands(op_equals(typed_value("foo", "long"), typed_value("foo", "long")), bool_value(true));
	assert check_operands(op_equals(typed_value("foo", "long"), typed_value("foo", "Long")), bool_value(false));
	assert check_operands(op_equals(iri_value("foo"), iri_value("foo")), bool_value(true));
	assert check_operands(op_equals(blank_value("foo"), blank_value("foo")), bool_value(true));
	
	assert check_operands(op_equals(int_value(3i64), bool_value(true)), error_value("=: expected numeric value but found bool_value(1)."));
	assert check_operands(op_equals(bool_value(true), unbound_value("foo")), error_value("=: ?foo was not bound."));
	assert check_operands(op_equals(bool_value(true), invalid_value("foo")), error_value("=: foo"));
	assert check_operands(op_equals(bool_value(true), error_value("foo")), error_value("=: foo"));
}

#[test]
fn operator_not_equals()
{
	assert check_operands(op_not_equals(bool_value(true), bool_value(true)), bool_value(false));
	assert check_operands(op_not_equals(bool_value(true), bool_value(false)), bool_value(true));
	
	assert check_operands(op_not_equals(int_value(3i64), int_value(3i64)), bool_value(false));
	assert check_operands(op_not_equals(int_value(3i64), int_value(4i64)), bool_value(true));
	assert check_operands(op_not_equals(int_value(3i64), float_value(4.0f64)), bool_value(true));
	assert check_operands(op_not_equals(int_value(3i64), float_value(3.0f64)), bool_value(false));
	assert check_operands(op_not_equals(float_value(3.0f64), float_value(3.0f64)), bool_value(false));
	
	assert check_operands(op_not_equals(string_value("foo", "en"), string_value("foo", "En")), bool_value(false));
	assert check_operands(op_not_equals(typed_value("foo", "long"), typed_value("foo", "long")), bool_value(false));
	assert check_operands(op_not_equals(typed_value("foo", "long"), typed_value("foo", "Long")), bool_value(true));
	assert check_operands(op_not_equals(iri_value("foo"), iri_value("foo")), bool_value(false));
	assert check_operands(op_not_equals(blank_value("foo"), blank_value("foo")), bool_value(false));
	
	assert check_operands(op_not_equals(int_value(3i64), bool_value(true)), error_value("!=: expected numeric value but found bool_value(1)."));
	assert check_operands(op_not_equals(bool_value(true), unbound_value("foo")), error_value("!=: ?foo was not bound."));
	assert check_operands(op_not_equals(bool_value(true), invalid_value("foo")), error_value("!=: foo"));
	assert check_operands(op_not_equals(bool_value(true), error_value("foo")), error_value("!=: foo"));
}

#[test]
fn operator_less_than()
{
	assert check_operands(op_less_than(int_value(3i64), int_value(3i64)), bool_value(false));
	assert check_operands(op_less_than(int_value(3i64), int_value(4i64)), bool_value(true));
	assert check_operands(op_less_than(int_value(3i64), float_value(4.0f64)), bool_value(true));
	assert check_operands(op_less_than(int_value(3i64), float_value(3.0f64)), bool_value(false));
	assert check_operands(op_less_than(float_value(3.0f64), float_value(3.0f64)), bool_value(false));
	
	assert check_operands(op_less_than(string_value("foo", "en"), string_value("goo", "en")), bool_value(true));
	assert check_operands(op_less_than(string_value("foo", "en"), string_value("aoo", "en")), bool_value(false));
	
	assert check_operands(op_less_than(bool_value(true), bool_value(true)), error_value("<: expected numeric, dateTime, string, or explicitly typed value but found bool_value(1)."));
	assert check_operands(op_less_than(iri_value("foo"), iri_value("foo")), error_value("<: expected numeric, dateTime, string, or explicitly typed value but found iri_value(\"foo\")."));
	assert check_operands(op_less_than(blank_value("foo"), blank_value("foo")), error_value("<: expected numeric, dateTime, string, or explicitly typed value but found blank_value(\"foo\")."));
	assert check_operands(op_less_than(int_value(3i64), bool_value(true)), error_value("<: expected numeric value but found bool_value(1)."));
}

#[test]
fn operator_other_relational()
{
	assert check_operands(op_greater_than(int_value(3i64), int_value(3i64)), bool_value(false));
	assert check_operands(op_less_than_or_equal(int_value(3i64), int_value(4i64)), bool_value(true));
	assert check_operands(op_greater_than_or_equal(int_value(3i64), float_value(4.0f64)), bool_value(false));
}

