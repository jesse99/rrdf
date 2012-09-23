use object::*;
use operators::*;
use test_helpers::*;

// Also tests get_ebv.
#[test]
fn operator_not()
{
	assert check_operands(op_not(InvalidValue(~"oops", ~"some:type")), BoolValue(true));

	assert check_operands(op_not(BoolValue(false)), BoolValue(true));
	
	assert check_operands(op_not(StringValue(~"hello", ~"en")), BoolValue(false));
	assert check_operands(op_not(TypedValue(~"", ~"my-type")), BoolValue(true));
	
	assert check_operands(op_not(IntValue(23i64)), BoolValue(false));
	assert check_operands(op_not(IntValue(0i64)), BoolValue(true));
	assert check_operands(op_not(FloatValue(2.3f64)), BoolValue(false));
	assert check_operands(op_not(FloatValue(0.0f64)), BoolValue(true));
	assert check_operands(op_not(FloatValue(f64::NaN)), BoolValue(true));
}

#[test]
fn operator_unary_minus()
{
	assert check_operands(op_unary_minus(IntValue(23i64)), IntValue(-23i64));
	assert check_operands(op_unary_minus(FloatValue(2.3f64)), FloatValue(-2.3f64));
	assert check_operands(op_unary_minus(StringValue(~"oops", ~"")), ErrorValue(~"unary minus: expected numeric value but found StringValue(~\"oops\", ~\"\")."));
}

#[test]
fn operator_or()
{
	assert check_operands(op_or(BoolValue(true), BoolValue(false)), BoolValue(true));
	assert check_operands(op_or(BoolValue(true), BoolValue(false)), BoolValue(true));
	
	assert check_operands(op_or(BoolValue(true), ErrorValue(~"oops.")), BoolValue(true));
	assert check_operands(op_or(ErrorValue(~"oops."), BoolValue(true)), BoolValue(true));
	
	assert check_operands(op_or(BoolValue(false), ErrorValue(~"oops.")), ErrorValue(~"oops."));
	assert check_operands(op_or(ErrorValue(~"oops."), BoolValue(false)), ErrorValue(~"oops."));
	
	assert check_operands(op_or(ErrorValue(~"ack."), ErrorValue(~"oops.")), ErrorValue(~"ack. oops."));
}

#[test]
fn operator_and()
{
	assert check_operands(op_and(BoolValue(true), BoolValue(true)), BoolValue(true));
	
	assert check_operands(op_and(BoolValue(true), BoolValue(false)), BoolValue(false));
	assert check_operands(op_and(BoolValue(true), BoolValue(false)), BoolValue(false));
	
	assert check_operands(op_and(BoolValue(true), ErrorValue(~"oops.")), ErrorValue(~"oops."));
	assert check_operands(op_and(ErrorValue(~"oops."), BoolValue(true)), ErrorValue(~"oops."));
	
	assert check_operands(op_and(BoolValue(false), ErrorValue(~"oops.")), BoolValue(false));
	assert check_operands(op_and(ErrorValue(~"oops."), BoolValue(false)), BoolValue(false));
	
	assert check_operands(op_and(ErrorValue(~"ack."), ErrorValue(~"oops.")), ErrorValue(~"ack. oops."));
}

#[test]
fn operator_equals()
{
	assert check_operands(op_equals(BoolValue(true), BoolValue(true)), BoolValue(true));
	assert check_operands(op_equals(BoolValue(true), BoolValue(false)), BoolValue(false));
	
	assert check_operands(op_equals(IntValue(3i64), IntValue(3i64)), BoolValue(true));
	assert check_operands(op_equals(IntValue(3i64), IntValue(4i64)), BoolValue(false));
	assert check_operands(op_equals(IntValue(3i64), FloatValue(4.0f64)), BoolValue(false));
	assert check_operands(op_equals(IntValue(3i64), FloatValue(3.0f64)), BoolValue(true));
	assert check_operands(op_equals(FloatValue(3.0f64), FloatValue(3.0f64)), BoolValue(true));
	
	assert check_operands(op_equals(StringValue(~"foo", ~"en"), StringValue(~"foo", ~"En")), BoolValue(true));
	assert check_operands(op_equals(TypedValue(~"foo", ~"long"), TypedValue(~"foo", ~"long")), BoolValue(true));
	assert check_operands(op_equals(TypedValue(~"foo", ~"long"), TypedValue(~"foo", ~"Long")), BoolValue(false));
	assert check_operands(op_equals(IriValue(~"foo"), IriValue(~"foo")), BoolValue(true));
	assert check_operands(op_equals(BlankValue(~"foo"), BlankValue(~"foo")), BoolValue(true)); 
	
	assert check_operands(op_equals(IntValue(3i64), BoolValue(true)), ErrorValue(~"=: expected numeric value but found BoolValue(true)."));
	assert check_operands(op_equals(BoolValue(true), UnboundValue(~"foo")), ErrorValue(~"=: ?foo was not bound."));
	assert check_operands(op_equals(BoolValue(true), InvalidValue(~"foo", ~"some:type")), ErrorValue(~"=: 'foo' is not a valid some:type"));
	assert check_operands(op_equals(BoolValue(true), ErrorValue(~"foo")), ErrorValue(~"=: foo"));
}

#[test]
fn operator_not_equals()
{
	assert check_operands(op_not_equals(BoolValue(true), BoolValue(true)), BoolValue(false));
	assert check_operands(op_not_equals(BoolValue(true), BoolValue(false)), BoolValue(true));
	
	assert check_operands(op_not_equals(IntValue(3i64), IntValue(3i64)), BoolValue(false));
	assert check_operands(op_not_equals(IntValue(3i64), IntValue(4i64)), BoolValue(true));
	assert check_operands(op_not_equals(IntValue(3i64), FloatValue(4.0f64)), BoolValue(true));
	assert check_operands(op_not_equals(IntValue(3i64), FloatValue(3.0f64)), BoolValue(false));
	assert check_operands(op_not_equals(FloatValue(3.0f64), FloatValue(3.0f64)), BoolValue(false));
	
	assert check_operands(op_not_equals(StringValue(~"foo", ~"en"), StringValue(~"foo", ~"En")), BoolValue(false));
	assert check_operands(op_not_equals(TypedValue(~"foo", ~"long"), TypedValue(~"foo", ~"long")), BoolValue(false));
	assert check_operands(op_not_equals(TypedValue(~"foo", ~"long"), TypedValue(~"foo", ~"Long")), BoolValue(true));
	assert check_operands(op_not_equals(IriValue(~"foo"), IriValue(~"foo")), BoolValue(false));
	assert check_operands(op_not_equals(BlankValue(~"foo"), BlankValue(~"foo")), BoolValue(false));
	
	assert check_operands(op_not_equals(IntValue(3i64), BoolValue(true)), ErrorValue(~"!=: expected numeric value but found BoolValue(true)."));
	assert check_operands(op_not_equals(BoolValue(true), UnboundValue(~"foo")), ErrorValue(~"!=: ?foo was not bound."));
	assert check_operands(op_not_equals(BoolValue(true), InvalidValue(~"foo", ~"some:type")), ErrorValue(~"!=: 'foo' is not a valid some:type"));
	assert check_operands(op_not_equals(BoolValue(true), ErrorValue(~"foo")), ErrorValue(~"!=: foo"));
}

#[test]
fn operator_less_than()
{
	assert check_operands(op_less_than(IntValue(3i64), IntValue(3i64)), BoolValue(false));
	assert check_operands(op_less_than(IntValue(3i64), IntValue(4i64)), BoolValue(true));
	assert check_operands(op_less_than(IntValue(3i64), FloatValue(4.0f64)), BoolValue(true));
	assert check_operands(op_less_than(IntValue(3i64), FloatValue(3.0f64)), BoolValue(false));
	assert check_operands(op_less_than(FloatValue(3.0f64), FloatValue(3.0f64)), BoolValue(false));
	
	assert check_operands(op_less_than(StringValue(~"foo", ~"en"), StringValue(~"goo", ~"en")), BoolValue(true));
	assert check_operands(op_less_than(StringValue(~"foo", ~"en"), StringValue(~"aoo", ~"en")), BoolValue(false));
	
	assert check_operands(op_less_than(IriValue(~"foo"), IriValue(~"foo")), BoolValue(false));
	assert check_operands(op_less_than(BlankValue(~"foo"), BlankValue(~"foo")), BoolValue(false));
	
	assert check_operands(op_less_than(BoolValue(true), BoolValue(true)), ErrorValue(~"<: expected numeric, dateTime, string, or explicitly typed value but found BoolValue(true)."));
	assert check_operands(op_less_than(IntValue(3i64), BoolValue(true)), ErrorValue(~"<: expected numeric value but found BoolValue(true)."));
}

#[test]
fn operator_other_relational()
{
	assert check_operands(op_greater_than(IntValue(3i64), IntValue(3i64)), BoolValue(false));
	assert check_operands(op_less_than_or_equal(IntValue(3i64), IntValue(4i64)), BoolValue(true));
	assert check_operands(op_greater_than_or_equal(IntValue(3i64), FloatValue(4.0f64)), BoolValue(false));
}

#[test]
fn operator_multiply()
{
	assert check_operands(op_multiply(IntValue(6i64), IntValue(2i64)), IntValue(12i64));
	assert check_operands(op_multiply(IntValue(3i64), FloatValue(4.0f64)), FloatValue(12.0f64));
	assert check_operands(op_multiply(IntValue(3i64), BoolValue(true)), ErrorValue(~"*: expected numeric value but found BoolValue(true)."));
}

#[test]
fn operator_other_arith()
{
	assert check_operands(op_divide(IntValue(6i64), IntValue(2i64)), IntValue(3i64));
	assert check_operands(op_divide(IntValue(3i64), IntValue(0i64)), ErrorValue(~"Divide by zero."));

	assert check_operands(op_add(IntValue(6i64), IntValue(2i64)), IntValue(8i64));
	assert check_operands(op_subtract(IntValue(6i64), IntValue(2i64)), IntValue(4i64));
}
