//! SPARQL functions. Clients will not ordinarily use this.
use expression::{expr, eval_expr};

fn bound_fn(operand: object) -> object
{
	match operand
	{
		unbound_value(_name) =>
		{
			bool_value(false)
		}
		_ =>
		{
			bool_value(true)
		}
	}
}

fn eval_if(context: query_context, bindings: ~[(~str, object)], args: ~[@expr]) -> object
{
	if vec::len(args) == 3u
	{
		let predicate = eval_expr(context, bindings, *args[0]);
		match get_ebv(predicate)
		{
			result::Ok(true) =>
			{
				eval_expr(context, bindings, *args[1])
			}
			result::Ok(false) =>
			{
				eval_expr(context, bindings, *args[2])
			}
			result::Err(err) =>
			{
				error_value(~"IF: " + err)
			}
		}
	}
	else
	{
		if vec::len(args) == 1u
		{
			error_value(~"IF accepts 3 arguments but was called with 1 argument.")
		}
		else
		{
			error_value(fmt!("IF accepts 3 arguments but was called with %? arguments.", vec::len(args)))
		}
	}
}

fn eval_coalesce(context: query_context, bindings: ~[(~str, object)], args: ~[@expr]) -> object
{
	for vec::each(args)
	|arg|
	{
		let candidate = eval_expr(context, bindings, *arg);
		match candidate
		{
			unbound_value(*) | invalid_value(*) | error_value(*) =>
			{
				// try the next argument
			}
			_ =>
			{
				return candidate;
			}
		}
	}
	
	return error_value(~"COALESCE: all arguments failed to evaluate");
}

fn sameterm_fn(lhs: object, rhs: object) -> object
{
	match lhs
	{
		bool_value(lvalue) =>
		{
			match rhs
			{
				bool_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		int_value(lvalue) =>
		{
			match rhs
			{
				int_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		float_value(lvalue) =>
		{
			match rhs
			{
				float_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		dateTime_value(lvalue) =>
		{
			match rhs
			{
				dateTime_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		string_value(lvalue, llang) =>
		{
			match rhs
			{
				string_value(rvalue, rlang) =>		// TODO: when we introduce type codes we'll need to check them here
				{
					bool_value(str::to_lower(llang) == str::to_lower(rlang) && lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		typed_value(lvalue, ltype) =>
		{
			match rhs
			{
				typed_value(rvalue, rtype) =>
				{
					bool_value(ltype == rtype && lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		iri_value(lvalue) =>
		{
			match rhs
			{
				iri_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		blank_value(lvalue) =>
		{
			match rhs
			{
				blank_value(rvalue) =>
				{
					bool_value(lvalue == rvalue)
				}
				_ =>
				{
					bool_value(false)
				}
			}
		}
		_ =>
		{
			bool_value(false)
		}
	}
}

// TODO: EXISTS and NOT EXISTS
