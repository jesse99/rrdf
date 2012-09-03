//! SPARQL FILTER expressions.
use functional_forms::{eval_if, eval_coalesce, bound_fn, sameterm_fn};
use functions_on_dates::*;
use functions_on_numerics::*;
use functions_on_strings::*;
use functions_on_terms::*;
use operators::*;

export expr, expr_to_str, eval_expr, constant_expr, variable_expr, call_expr, extension_expr;

enum expr
{
	constant_expr(object),
	variable_expr(~str),
	call_expr(~str, ~[@expr]),			// function name + arguments
	extension_expr(~str, ~[@expr])	// function name + arguments
}

fn expr_to_str(store: store, expr: expr) -> ~str
{
	match expr
	{
		constant_expr(o) =>
		{
			object_to_str(store, o)
		}
		variable_expr(v) =>
		{
			fmt!("?%s", v)
		}
		call_expr(n, args) | extension_expr(n, args) =>
		{
			n + str::connect(do args.map |a| {expr_to_str(store, *a)}, ~", ")
		}
	}
}

fn eval_expr(context: query_context, bindings: ~[(~str, object)], expr: expr) -> object
{
	let result = match expr
	{
		constant_expr(value) =>
		{
			value
		}
		variable_expr(name) =>
		{
			match bindings.search(name)
			{
				option::Some(value) =>
				{
					value
				}
				option::None =>
				{
					unbound_value(name)
				}
			}
		}
		extension_expr(fname, args) =>
		{
			eval_extension(context, bindings, fname, args)
		}
		call_expr(~"if_fn", args) =>				// special case this because it is supposed to short circuit
		{
			eval_if(context, bindings, args)
		}
		call_expr(~"coalesce_fn", args) =>		// special case this because it is variadic
		{
			eval_coalesce(context, bindings, args)
		}
		call_expr(fname, args) =>
		{
			eval_call(context, bindings, fname, args)
		}
	};
	
	debug!("Eval %? = %s", expr, result.to_str());
	return result;
}

// ---- Internal Functions ----------------------------------------------------
type unary_fn = fn (object) -> object;
type binary_fn = fn (object, object) -> object;
type ternary_fn = fn (object, object, object) -> object;

fn eval_extension(context: query_context, bindings: ~[(~str, object)], fname: ~str, args: ~[@expr]) -> object
{
	let args = do vec::map(args) |a| {eval_expr(context, bindings, *a)};		// note that we want to call the function even if we get errors here because some functions are OK with them
	match context.extensions.find(fname)
	{
		option::Some(f) =>
		{
			f(context.namespaces, args)
		}
		option::None =>
		{
			error_value(fmt!("%s wasn't registered with the store as an extension function", fname))
		}
	}
}

fn eval_call(context: query_context, bindings: ~[(~str, object)], fname: ~str, args: ~[@expr]) -> object
{
	let args = do vec::map(args) |a| {eval_expr(context, bindings, *a)};		// note that we want to call the function even if we get errors here because some functions are OK with them
	match fname
	{
		// operators
		~"op_not" =>										// macros currently must be expressions so we can't use them here
		{
			eval_call1(fname, @op_not, args)
		}
		~"op_unary_plus" =>
		{
			eval_call1(fname, @op_unary_plus, args)
		}
		~"op_unary_minus" =>
		{
			eval_call1(fname, @op_unary_minus, args)
		}
		~"op_or" =>
		{
			eval_call2(fname, @op_or, args)
		}
		~"op_and" =>
		{
			eval_call2(fname, @op_and, args)
		}
		~"op_equals" =>
		{
			eval_call2(fname, @op_equals, args)
		}
		~"op_not_equals" =>
		{
			eval_call2(fname, @op_not_equals, args)
		}
		~"op_less_than" =>
		{
			eval_call2(fname, @op_less_than, args)
		}
		~"op_less_than_or_equal" =>
		{
			eval_call2(fname, @op_less_than_or_equal, args)
		}
		~"op_greater_than" =>
		{
			eval_call2(fname, @op_greater_than, args)
		}
		~"op_greater_than_or_equal" =>
		{
			eval_call2(fname, @op_greater_than_or_equal, args)
		}
		~"op_multiply" =>
		{
			eval_call2(fname, @op_multiply, args)
		}
		~"op_divide" =>
		{
			eval_call2(fname, @op_divide, args)
		}
		~"op_add" =>
		{
			eval_call2(fname, @op_add, args)
		}
		~"op_subtract" =>
		{
			eval_call2(fname, @op_subtract, args)
		}
		// functional forms
		~"bound_fn" =>
		{
			eval_call1(fname, @bound_fn, args)
		}
		~"sameterm_fn" =>
		{
			eval_call2(fname, @sameterm_fn, args)
		}
		// functions on terms
		~"isiri_fn" =>
		{
			eval_call1(fname, @isiri_fn, args)
		}
		~"isblank_fn" =>
		{
			eval_call1(fname, @isblank_fn, args)
		}
		~"isliteral_fn" =>
		{
			eval_call1(fname, @isliteral_fn, args)
		}
		~"isnumeric_fn" =>
		{
			eval_call1(fname, @isnumeric_fn, args)
		}
		~"str_fn" =>
		{
			eval_call1(fname, @str_fn, args)
		}
		~"lang_fn" =>
		{
			eval_call1(fname, @lang_fn, args)
		}
		~"datatype_fn" =>
		{
			eval_call1(fname, @datatype_fn, args)
		}
		~"strdt_fn" =>
		{
			eval_call2(fname, @strdt_fn, args)
		}
		~"strlang_fn" =>
		{
			eval_call2(fname, @strlang_fn, args)
		}
		// functions on strings
		~"strlen_fn" =>
		{
			eval_call1(fname, @strlen_fn, args)
		}
		~"substr2_fn" =>
		{
			eval_call2(fname, @substr2_fn, args)
		}
		~"substr3_fn" =>
		{
			eval_call3(fname, @substr3_fn, args)
		}
		~"ucase_fn" =>
		{
			eval_call1(fname, @ucase_fn, args)
		}
		~"lcase_fn" =>
		{
			eval_call1(fname, @lcase_fn, args)
		}
		~"strstarts_fn" =>
		{
			eval_call2(fname, @strstarts_fn, args)
		}
		~"strends_fn" =>
		{
			eval_call2(fname, @strends_fn, args)
		}
		~"contains_fn" =>
		{
			eval_call2(fname, @contains_fn, args)
		}
		~"strbefore_fn" =>
		{
			eval_call2(fname, @strbefore_fn, args)
		}
		~"strafter_fn" =>
		{
			eval_call2(fname, @strafter_fn, args)
		}
		~"encode_for_uri_fn" =>
		{
			eval_call1(fname, @encode_for_uri_fn, args)
		}
		~"concat_fn" =>
		{
			concat_fn(args)
		}
		~"langmatches_fn" =>
		{
			eval_call2(fname, @langmatches_fn, args)
		}
		// functions on numerics
		~"abs_fn" =>
		{
			eval_call1(fname, @abs_fn, args)
		}
		~"round_fn" =>
		{
			eval_call1(fname, @round_fn, args)
		}
		~"ceil_fn" =>
		{
			eval_call1(fname, @ceil_fn, args)
		}
		~"floor_fn" =>
		{
			eval_call1(fname, @floor_fn, args)
		}
		~"rand_fn" =>
		{
			rand_fn(context, args)
		}
		// functions on dates
		~"now_fn" =>
		{
			now_fn(context, args)
		}
		~"year_fn" =>
		{
			eval_call1(fname, @year_fn, args)
		}
		~"month_fn" =>
		{
			eval_call1(fname, @month_fn, args)
		}
		~"day_fn" =>
		{
			eval_call1(fname, @day_fn, args)
		}
		~"hours_fn" =>
		{
			eval_call1(fname, @hours_fn, args)
		}
		~"minutes_fn" =>
		{
			eval_call1(fname, @minutes_fn, args)
		}
		~"seconds_fn" =>
		{
			eval_call1(fname, @seconds_fn, args)
		}
		~"tz_fn" =>
		{
			eval_call1(fname, @tz_fn, args)
		}
		// unknown functions
		_ =>
		{
			error_value(fmt!("%s is not implemented.", fname))
		}
	}
}

fn eval_call1(fname: ~str, fp: @unary_fn, args: ~[object]) -> object
{
	if vec::len(args) == 1u
	{
		(*fp)(args[0])
	}
	else
	{
		error_value(fmt!("%s accepts 1 argument but was called with %? arguments.", fname, vec::len(args)))
	}
}

fn eval_call2(fname: ~str, fp: @binary_fn, args: ~[object]) -> object
{
	if vec::len(args) == 2u
	{
		(*fp)(args[0], args[1])
	}
	else
	{
		if vec::len(args) == 1u
		{
			error_value(fmt!("%s accepts 2 arguments but was called with 1 argument.", fname))
		}
		else
		{
			error_value(fmt!("%s accepts 2 arguments but was called with %? arguments.", fname, vec::len(args)))
		}
	}
}

fn eval_call3(fname: ~str, fp: @ternary_fn, args: ~[object]) -> object
{
	if vec::len(args) == 3u
	{
		(*fp)(args[0], args[1], args[2])
	}
	else
	{
		if vec::len(args) == 1u
		{
			error_value(fmt!("%s accepts 3 arguments but was called with 1 argument.", fname))
		}
		else
		{
			error_value(fmt!("%s accepts 3 arguments but was called with %? arguments.", fname, vec::len(args)))
		}
	}
}

