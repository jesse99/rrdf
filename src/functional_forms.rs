//! SPARQL functions. Clients will not ordinarily use this.

pub pure fn bound_fn(operand: &Object) -> Object
{
	match *operand
	{
		UnboundValue =>
		{
			BoolValue(false)
		}
		_ =>
		{
			BoolValue(true)
		}
	}
}

pub pure fn eval_if(context: &QueryContext, solution: &Solution, row: &SolutionRow, args: &~[@expression::Expr]) -> @Object
{
	if args.len() == 3u
	{
		let predicate = expression::eval_expr(context, solution, row, args[0]);
		match get_ebv(predicate)
		{
			result::Ok(true) =>
			{
				expression::eval_expr(context, solution, row, args[1])
			}
			result::Ok(false) =>
			{
				expression::eval_expr(context, solution, row, args[2])
			}
			result::Err(copy err) =>
			{
				@ErrorValue(~"IF: " + err)
			}
		}
	}
	else
	{
		if args.len() == 1u
		{
			@ErrorValue(~"IF accepts 3 arguments but was called with 1 argument.")
		}
		else
		{
			@ErrorValue(fmt!("IF accepts 3 arguments but was called with %? arguments.", args.len()))
		}
	}
}

pub pure fn eval_coalesce(context: &QueryContext, solution: &Solution, row: &SolutionRow, args: &~[@expression::Expr]) -> @Object
{
	for vec::each(*args)
	|arg|
	{
		let candidate = expression::eval_expr(context, solution, row, *arg);
		match *candidate
		{
			UnboundValue | InvalidValue(*) | ErrorValue(*) =>
			{
				// try the next argument
			}
			_ =>
			{
				return candidate;
			}
		}
	}
	
	return @ErrorValue(~"COALESCE: all arguments failed to evaluate");
}

pub pure fn sameterm_fn(lhs: &Object, rhs: &Object) -> Object
{
	match *lhs
	{
		BoolValue(lvalue) =>
		{
			match *rhs
			{
				BoolValue(rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		IntValue(lvalue) =>
		{
			match *rhs
			{
				IntValue(rvalue) =>
				{
					BoolValue(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		FloatValue(lvalue) =>
		{
			match *rhs
			{
				FloatValue(rvalue) =>
				{
					BoolValue(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		DateTimeValue(ref lvalue) =>
		{
			match *rhs
			{
				DateTimeValue(ref rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		StringValue(ref lvalue, ref llang) =>
		{
			match *rhs
			{
				StringValue(ref rvalue, ref rlang) =>		// TODO: when we introduce type codes we'll need to check them here
				{
					BoolValue(str::to_lower(*llang) == str::to_lower(*rlang) && lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		TypedValue(ref lvalue, ref ltype) =>
		{
			match *rhs
			{
				TypedValue(ref rvalue, ref rtype) =>
				{
					BoolValue(ltype == rtype && lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		IriValue(ref lvalue) =>
		{
			match *rhs
			{
				IriValue(ref rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		BlankValue(ref lvalue) =>
		{
			match *rhs
			{
				BlankValue(ref rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		_ =>
		{
			BoolValue(false)
		}
	}
}

// TODO: EXISTS and NOT EXISTS
