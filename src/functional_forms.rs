#[doc = "SPARQL functions. Clients will not ordinarily use this."];

fn bound_fn(operand: object) -> object
{
	alt operand
	{
		unbound_value(_name)
		{
			bool_value(false)
		}
		_
		{
			bool_value(true)
		}
	}
}

// eval_expr special cases if_fn and coalesce

// TODO: implement NOT EXISTS and EXISTS (these take patterns, not expressions)

fn sameterm_fn(lhs: object, rhs: object) -> object
{
	alt lhs
	{
		bool_value(lvalue)
		{
			alt rhs
			{
				bool_value(rvalue)
				{
					bool_value(lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		int_value(lvalue)
		{
			alt rhs
			{
				int_value(rvalue)
				{
					bool_value(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_
				{
					bool_value(false)
				}
			}
		}
		float_value(lvalue)
		{
			alt rhs
			{
				float_value(rvalue)
				{
					bool_value(lvalue == rvalue)	// TODO: when we introduce type codes we'll need to check them here
				}
				_
				{
					bool_value(false)
				}
			}
		}
		dateTime_value(lvalue)
		{
			alt rhs
			{
				dateTime_value(rvalue)
				{
					bool_value(lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		string_value(lvalue, llang)
		{
			alt rhs
			{
				string_value(rvalue, rlang)		// TODO: when we introduce type codes we'll need to check them here
				{
					bool_value(str::to_lower(llang) == str::to_lower(rlang) && lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		typed_value(lvalue, ltype)
		{
			alt rhs
			{
				typed_value(rvalue, rtype)
				{
					bool_value(ltype == rtype && lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		iri_value(lvalue)
		{
			alt rhs
			{
				iri_value(rvalue)
				{
					bool_value(lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		blank_value(lvalue)
		{
			alt rhs
			{
				blank_value(rvalue)
				{
					bool_value(lvalue == rvalue)
				}
				_
				{
					bool_value(false)
				}
			}
		}
		_
		{
			bool_value(false)
		}
	}
}
