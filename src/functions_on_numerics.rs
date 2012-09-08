//! SPARQL functions. Clients will not ordinarily use this.
use store::*;
use object::*;

fn abs_fn(operand: Object) -> Object
{
	match operand
	{
		IntValue(value) =>
		{
			IntValue(i64::abs(value))
		}
		FloatValue(value) =>
		{
			FloatValue(f64::abs(value))
		}
		_ =>
		{
			ErrorValue(fmt!("ABS: expected numeric but found %?.", operand))
		}
	}
}

fn round_fn(operand: Object) -> Object
{
	match operand
	{
		IntValue(_) =>
		{
			operand
		}
		FloatValue(value) =>
		{
			FloatValue(f64::round(value))
		}
		_ =>
		{
			ErrorValue(fmt!("ROUND: expected numeric but found %?.", operand))
		}
	}
}

fn ceil_fn(operand: Object) -> Object
{
	match operand
	{
		IntValue(_) =>
		{
			operand
		}
		FloatValue(value) =>
		{
			FloatValue(f64::ceil(value))
		}
		_ =>
		{
			ErrorValue(fmt!("CEIL: expected numeric but found %?.", operand))
		}
	}
}

fn floor_fn(operand: Object) -> Object
{
	match operand
	{
		IntValue(_) =>
		{
			operand
		}
		FloatValue(value) =>
		{
			FloatValue(f64::floor(value))
		}
		_ =>
		{
			ErrorValue(fmt!("FLOOR: expected numeric but found %?.", operand))
		}
	}
}

fn rand_fn(context: &query::QueryContext, args: ~[Object]) -> Object
{
	if vec::len(args) == 0u
	{
		let n = context.rng.next() as f64;
		let d = u32::max_value as f64;
		FloatValue(n/d)
	}
	else
	{
		ErrorValue(fmt!("RAND accepts 0 arguments but was called with %? arguments.", vec::len(args)))
	}
}
