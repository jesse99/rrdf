#[doc = "SPARQL functions. Clients will not ordinarily use this."];

fn abs_fn(operand: object) -> object
{
	alt operand
	{
		int_value(value)
		{
			int_value(i64::abs(value))
		}
		float_value(value)
		{
			float_value(f64::abs(value))
		}
		_
		{
			error_value(#fmt["ABS: expected numeric but found %?.", operand])
		}
	}
}

fn round_fn(operand: object) -> object
{
	alt operand
	{
		int_value(value)
		{
			operand
		}
		float_value(value)
		{
			float_value(f64::round(value))
		}
		_
		{
			error_value(#fmt["ROUND: expected numeric but found %?.", operand])
		}
	}
}

fn ceil_fn(operand: object) -> object
{
	alt operand
	{
		int_value(value)
		{
			operand
		}
		float_value(value)
		{
			float_value(f64::ceil(value))
		}
		_
		{
			error_value(#fmt["CEIL: expected numeric but found %?.", operand])
		}
	}
}

fn floor_fn(operand: object) -> object
{
	alt operand
	{
		int_value(value)
		{
			operand
		}
		float_value(value)
		{
			float_value(f64::floor(value))
		}
		_
		{
			error_value(#fmt["FLOOR: expected numeric but found %?.", operand])
		}
	}
}

// TODO: add rand
