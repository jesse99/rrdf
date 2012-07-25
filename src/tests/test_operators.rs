//import operands::*;
import operators::*;
import test_helpers::*;

// Also tests get_ebv.
#[test]
fn operator_not()
{
	assert check_operands(op_not(invalid_value(~"oops", ~"some:type")), bool_value(true));

	assert check_operands(op_not(bool_value(false)), bool_value(true));
	
	assert check_operands(op_not(string_value(~"hello", ~"en")), bool_value(false));
	assert check_operands(op_not(typed_value(~"", ~"my-type")), bool_value(true));
	
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
	assert check_operands(op_unary_minus(string_value(~"oops", ~"")), error_value(~"unary minus: expected numeric value but found string_value(~\"oops\", ~\"\")."));
}

#[test]
fn operator_or()
{
	assert check_operands(op_or(bool_value(true), bool_value(false)), bool_value(true));
	assert check_operands(op_or(bool_value(true), bool_value(false)), bool_value(true));
	
	assert check_operands(op_or(bool_value(true), error_value(~"oops.")), bool_value(true));
	assert check_operands(op_or(error_value(~"oops."), bool_value(true)), bool_value(true));
	
	assert check_operands(op_or(bool_value(false), error_value(~"oops.")), error_value(~"oops."));
	assert check_operands(op_or(error_value(~"oops."), bool_value(false)), error_value(~"oops."));
	
	assert check_operands(op_or(error_value(~"ack."), error_value(~"oops.")), error_value(~"ack. oops."));
}

#[test]
fn operator_and()
{
	assert check_operands(op_and(bool_value(true), bool_value(true)), bool_value(true));
	
	assert check_operands(op_and(bool_value(true), bool_value(false)), bool_value(false));
	assert check_operands(op_and(bool_value(true), bool_value(false)), bool_value(false));
	
	assert check_operands(op_and(bool_value(true), error_value(~"oops.")), error_value(~"oops."));
	assert check_operands(op_and(error_value(~"oops."), bool_value(true)), error_value(~"oops."));
	
	assert check_operands(op_and(bool_value(false), error_value(~"oops.")), bool_value(false));
	assert check_operands(op_and(error_value(~"oops."), bool_value(false)), bool_value(false));
	
	assert check_operands(op_and(error_value(~"ack."), error_value(~"oops.")), error_value(~"ack. oops."));
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
	
	assert check_operands(op_equals(string_value(~"foo", ~"en"), string_value(~"foo", ~"En")), bool_value(true));
	assert check_operands(op_equals(typed_value(~"foo", ~"long"), typed_value(~"foo", ~"long")), bool_value(true));
	assert check_operands(op_equals(typed_value(~"foo", ~"long"), typed_value(~"foo", ~"Long")), bool_value(false));
	assert check_operands(op_equals(iri_value(~"foo"), iri_value(~"foo")), bool_value(true));
	assert check_operands(op_equals(blank_value(~"foo"), blank_value(~"foo")), bool_value(true)); 
	
	assert check_operands(op_equals(int_value(3i64), bool_value(true)), error_value(~"=: expected numeric value but found bool_value(1)."));
	assert check_operands(op_equals(bool_value(true), unbound_value(~"foo")), error_value(~"=: ?foo was not bound."));
	assert check_operands(op_equals(bool_value(true), invalid_value(~"foo", ~"some:type")), error_value(~"=: 'foo' is not a valid some:type"));
	assert check_operands(op_equals(bool_value(true), error_value(~"foo")), error_value(~"=: foo"));
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
	
	assert check_operands(op_not_equals(string_value(~"foo", ~"en"), string_value(~"foo", ~"En")), bool_value(false));
	assert check_operands(op_not_equals(typed_value(~"foo", ~"long"), typed_value(~"foo", ~"long")), bool_value(false));
	assert check_operands(op_not_equals(typed_value(~"foo", ~"long"), typed_value(~"foo", ~"Long")), bool_value(true));
	assert check_operands(op_not_equals(iri_value(~"foo"), iri_value(~"foo")), bool_value(false));
	assert check_operands(op_not_equals(blank_value(~"foo"), blank_value(~"foo")), bool_value(false));
	
	assert check_operands(op_not_equals(int_value(3i64), bool_value(true)), error_value(~"!=: expected numeric value but found bool_value(1)."));
	assert check_operands(op_not_equals(bool_value(true), unbound_value(~"foo")), error_value(~"!=: ?foo was not bound."));
	assert check_operands(op_not_equals(bool_value(true), invalid_value(~"foo", ~"some:type")), error_value(~"!=: 'foo' is not a valid some:type"));
	assert check_operands(op_not_equals(bool_value(true), error_value(~"foo")), error_value(~"!=: foo"));
}

#[test]
fn operator_less_than()
{
	assert check_operands(op_less_than(int_value(3i64), int_value(3i64)), bool_value(false));
	assert check_operands(op_less_than(int_value(3i64), int_value(4i64)), bool_value(true));
	assert check_operands(op_less_than(int_value(3i64), float_value(4.0f64)), bool_value(true));
	assert check_operands(op_less_than(int_value(3i64), float_value(3.0f64)), bool_value(false));
	assert check_operands(op_less_than(float_value(3.0f64), float_value(3.0f64)), bool_value(false));
	
	assert check_operands(op_less_than(string_value(~"foo", ~"en"), string_value(~"goo", ~"en")), bool_value(true));
	assert check_operands(op_less_than(string_value(~"foo", ~"en"), string_value(~"aoo", ~"en")), bool_value(false));
	
	assert check_operands(op_less_than(iri_value(~"foo"), iri_value(~"foo")), bool_value(false));
	assert check_operands(op_less_than(blank_value(~"foo"), blank_value(~"foo")), bool_value(false));
	
	assert check_operands(op_less_than(bool_value(true), bool_value(true)), error_value(~"<: expected numeric, dateTime, string, or explicitly typed value but found bool_value(1)."));
	assert check_operands(op_less_than(int_value(3i64), bool_value(true)), error_value(~"<: expected numeric value but found bool_value(1)."));
}

#[test]
fn operator_other_relational()
{
	assert check_operands(op_greater_than(int_value(3i64), int_value(3i64)), bool_value(false));
	assert check_operands(op_less_than_or_equal(int_value(3i64), int_value(4i64)), bool_value(true));
	assert check_operands(op_greater_than_or_equal(int_value(3i64), float_value(4.0f64)), bool_value(false));
}

#[test]
fn operator_multiply()
{
	assert check_operands(op_multiply(int_value(6i64), int_value(2i64)), int_value(12i64));
	assert check_operands(op_multiply(int_value(3i64), float_value(4.0f64)), float_value(12.0f64));
	assert check_operands(op_multiply(int_value(3i64), bool_value(true)), error_value(~"*: expected numeric value but found bool_value(1)."));
}

#[test]
fn operator_other_arith()
{
	assert check_operands(op_divide(int_value(6i64), int_value(2i64)), int_value(3i64));
	assert check_operands(op_divide(int_value(3i64), int_value(0i64)), error_value(~"Divide by zero."));

	assert check_operands(op_add(int_value(6i64), int_value(2i64)), int_value(8i64));
	assert check_operands(op_subtract(int_value(6i64), int_value(2i64)), int_value(4i64));
}
