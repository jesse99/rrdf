//! SPARQL functions. Clients will not ordinarily use this.
use store::*;
use object::*;

fn bound_fn(operand: Object) -> Object
{
	match operand
	{
		UnboundValue(_name) =>
		{
			BoolValue(false)
		}
		_ =>
		{
			BoolValue(true)
		}
	}
}

fn eval_if(context: &query::QueryContext, bindings: ~[(~str, Object)], args: ~[@expression::Expr]) -> Object
{
	if vec::len(args) == 3u
	{
		let predicate = expression::eval_expr(context, bindings, *args[0]);
		match get_ebv(predicate)
		{
			result::Ok(true) =>
			{
				expression::eval_expr(context, bindings, *args[1])
			}
			result::Ok(false) =>
			{
				expression::eval_expr(context, bindings, *args[2])
			}
			result::Err(err) =>
			{
				ErrorValue(~"IF: " + err)
			}
		}
	}
	else
	{
		if vec::len(args) == 1u
		{
			ErrorValue(~"IF accepts 3 arguments but was called with 1 argument.")
		}
		else
		{
			ErrorValue(fmt!("IF accepts 3 arguments but was called with %? arguments.", vec::len(args)))
		}
	}
}

fn eval_coalesce(context: &query::QueryContext, bindings: ~[(~str, Object)], args: ~[@expression::Expr]) -> Object
{
	for vec::each(args)
	|arg|
	{
		let candidate = expression::eval_expr(context, bindings, **arg);
		match candidate
		{
			UnboundValue(*) | InvalidValue(*) | ErrorValue(*) =>
			{
				// try the next argument
			}
			_ =>
			{
				return candidate;
			}
		}
	}
	
	return ErrorValue(~"COALESCE: all arguments failed to evaluate");
}

fn sameterm_fn(lhs: Object, rhs: Object) -> Object
{
	match lhs
	{
		BoolValue(lvalue) =>
		{
			match rhs
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
			match rhs
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
			match rhs
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
		DateTimeValue(lvalue) =>
		{
			match rhs
			{
				DateTimeValue(rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		StringValue(lvalue, llang) =>
		{
			match rhs
			{
				StringValue(rvalue, rlang) =>		// TODO: when we introduce type codes we'll need to check them here
				{
					BoolValue(str::to_lower(llang) == str::to_lower(rlang) && lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		TypedValue(lvalue, ltype) =>
		{
			match rhs
			{
				TypedValue(rvalue, rtype) =>
				{
					BoolValue(ltype == rtype && lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		IriValue(lvalue) =>
		{
			match rhs
			{
				IriValue(rvalue) =>
				{
					BoolValue(lvalue == rvalue)
				}
				_ =>
				{
					BoolValue(false)
				}
			}
		}
		BlankValue(lvalue) =>
		{
			match rhs
			{
				BlankValue(rvalue) =>
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
