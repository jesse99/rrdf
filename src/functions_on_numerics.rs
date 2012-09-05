//! SPARQL functions. Clients will not ordinarily use this.
use store::*;
use object::*;

fn abs_fn(operand: object) -> object
{
	match operand
	{
		int_value(value) =>
		{
			int_value(i64::abs(value))
		}
		float_value(value) =>
		{
			float_value(f64::abs(value))
		}
		_ =>
		{
			error_value(fmt!("ABS: expected numeric but found %?.", operand))
		}
	}
}

fn round_fn(operand: object) -> object
{
	match operand
	{
		int_value(_) =>
		{
			operand
		}
		float_value(value) =>
		{
			float_value(f64::round(value))
		}
		_ =>
		{
			error_value(fmt!("ROUND: expected numeric but found %?.", operand))
		}
	}
}

fn ceil_fn(operand: object) -> object
{
	match operand
	{
		int_value(_) =>
		{
			operand
		}
		float_value(value) =>
		{
			float_value(f64::ceil(value))
		}
		_ =>
		{
			error_value(fmt!("CEIL: expected numeric but found %?.", operand))
		}
	}
}

fn floor_fn(operand: object) -> object
{
	match operand
	{
		int_value(_) =>
		{
			operand
		}
		float_value(value) =>
		{
			float_value(f64::floor(value))
		}
		_ =>
		{
			error_value(fmt!("FLOOR: expected numeric but found %?.", operand))
		}
	}
}

fn rand_fn(context: query_context, args: ~[object]) -> object
{
	if vec::len(args) == 0u
	{
		let n = context.rng.next() as f64;
		let d = u32::max_value as f64;
		float_value(n/d)
	}
	else
	{
		error_value(fmt!("RAND accepts 0 arguments but was called with %? arguments.", vec::len(args)))
	}
}
