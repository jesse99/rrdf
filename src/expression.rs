#[doc = "SPARQL FILTER expressions."];
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
	alt expr
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
		call_expr(fname, args)
		{
			eval_call(bindings, fname, args)
		}
	}
}

// ---- Internal Functions ----------------------------------------------------
type unary_fn = fn (object) -> object;
type binary_fn = fn (object, object) -> object;

fn eval_call(bindings: [(str, object)], fname: str, args: [@expr]) -> object
{
	let args = vec::map(args) {|a| eval_expr(bindings, *a)};		// note that we want to call the function even if we get errors here because some functions are OK with them
	alt fname
	{
		// unary operators
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
		// binary operators
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

