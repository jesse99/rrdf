#[doc = "SPARQL FILTER expressions."];
import functional_forms::*;
import operators::*;

export expr, eval_expr;

enum expr
{
	constant_expr(object),
	variable_expr(str),
	call_expr(str, [@expr])		// function name + arguments
}

fn eval_expr(bindings: [(str, object)], expr: expr) -> object
{
	let result = alt expr
	{
		constant_expr(value)
		{
			value
		}
		variable_expr(name)
		{
			alt bindings.search(name)
			{
				option::some(value)
				{
					value
				}
				option::none
				{
					unbound_value(name)
				}
			}
		}
		call_expr("if_fn", args)				// special case this because it is supposed to short circuit
		{
			eval_if(bindings, args)
		}
		call_expr("coalesce_fn", args)		// special case this because it is variadic
		{
			eval_coalesce(bindings, args)
		}
		call_expr(fname, args)
		{
			eval_call(bindings, fname, args)
		}
	};
	
	#debug["Eval %? = %s", expr, result.to_str()];
	ret result;
}

// ---- Internal Functions ----------------------------------------------------
type unary_fn = fn (object) -> object;
type binary_fn = fn (object, object) -> object;

fn eval_if(bindings: [(str, object)], args: [@expr]) -> object
{
	if vec::len(args) == 3u
	{
		let predicate = eval_expr(bindings, *args[0]);
		alt get_ebv(predicate)
		{
			result::ok(true)
			{
				eval_expr(bindings, *args[1])
			}
			result::ok(false)
			{
				eval_expr(bindings, *args[2])
			}
			result::err(err)
			{
				error_value("IF: " + err)
			}
		}
	}
	else
	{
		if vec::len(args) == 1u
		{
			error_value("IF accepts 3 arguments but was called with 1 argument.")
		}
		else
		{
			error_value(#fmt["IF accepts 3 arguments but was called with %? arguments.", vec::len(args)])
		}
	}
}

fn eval_coalesce(bindings: [(str, object)], args: [@expr]) -> object
{
	for vec::each(args)
	{|arg|
		let candidate = eval_expr(bindings, *arg);
		alt candidate
		{
			unbound_value(*) | invalid_value(*) | error_value(*)
			{
				// try the next argument
			}
			_
			{
				ret candidate;
			}
		}
	}
	
	ret error_value("COALESCE: all arguments failed to evaluate");
}

fn eval_call(bindings: [(str, object)], fname: str, args: [@expr]) -> object
{
	let args = vec::map(args) {|a| eval_expr(bindings, *a)};		// note that we want to call the function even if we get errors here because some functions are OK with them
	alt fname
	{
		// operators
		"op_not"										// macros currently must be expressions so we can't use them here
		{
			eval_call1(fname, @op_not, args)
		}
		"op_unary_plus"
		{
			eval_call1(fname, @op_unary_plus, args)
		}
		"op_unary_minus"
		{
			eval_call1(fname, @op_unary_minus, args)
		}
		"op_or"
		{
			eval_call2(fname, @op_or, args)
		}
		"op_and"
		{
			eval_call2(fname, @op_and, args)
		}
		"op_equals"
		{
			eval_call2(fname, @op_equals, args)
		}
		"op_not_equals"
		{
			eval_call2(fname, @op_not_equals, args)
		}
		"op_less_than"
		{
			eval_call2(fname, @op_less_than, args)
		}
		"op_less_than_or_equal"
		{
			eval_call2(fname, @op_less_than_or_equal, args)
		}
		"op_greater_than"
		{
			eval_call2(fname, @op_greater_than, args)
		}
		"op_greater_than_or_equal"
		{
			eval_call2(fname, @op_greater_than_or_equal, args)
		}
		"op_multiply"
		{
			eval_call2(fname, @op_multiply, args)
		}
		"op_divide"
		{
			eval_call2(fname, @op_divide, args)
		}
		"op_add"
		{
			eval_call2(fname, @op_add, args)
		}
		"op_subtract"
		{
			eval_call2(fname, @op_subtract, args)
		}
		// functional forms
		"bound_fn"
		{
			eval_call1(fname, @bound_fn, args)
		}
		"sameterm_fn"
		{
			eval_call2(fname, @sameterm_fn, args)
		}
		// TODO: note that if_fn and coalesce_fn take exprs
		// unknown functions
		_
		{
			error_value(#fmt["%s is not implemented.", fname])
		}
	}
}

fn eval_call1(fname: str, fp: @unary_fn, args: [object]) -> object
{
	if vec::len(args) == 1u
	{
		(*fp)(args[0])
	}
	else
	{
		error_value(#fmt["%s accepts 1 argument but was called with %? arguments.", fname, vec::len(args)])
	}
}

fn eval_call2(fname: str, fp: @binary_fn, args: [object]) -> object
{
	if vec::len(args) == 2u
	{
		(*fp)(args[0], args[1])
	}
	else
	{
		if vec::len(args) == 1u
		{
			error_value(#fmt["%s accepts 2 arguments but was called with 1 argument.", fname])
		}
		else
		{
			error_value(#fmt["%s accepts 2 arguments but was called with %? arguments.", fname, vec::len(args)])
		}
	}
}

