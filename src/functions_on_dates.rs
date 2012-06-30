#[doc = "SPARQL functions. Clients will not ordinarily use this."];

fn now_fn(context: query_context, args: [object]) -> object
{
	if vec::len(args) == 0u
	{
		dateTime_value(context.timestamp)
	}
	else
	{
		error_value(#fmt["NOW accepts 0 arguments but was called with %? arguments.", vec::len(args)])
	}
}

fn year_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value((1900i32 + value.tm_year) as i64)
		}
		_
		{
			error_value(#fmt["YEAR: expected dateTime but found %?.", operand])
		}
	}
}

fn month_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value((1i32 + value.tm_mon) as i64)
		}
		_
		{
			error_value(#fmt["MONTH: expected dateTime but found %?.", operand])
		}
	}
}

fn day_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value(value.tm_mday as i64)
		}
		_
		{
			error_value(#fmt["DAY: expected dateTime but found %?.", operand])
		}
	}
}

fn hours_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value(value.tm_hour as i64)
		}
		_
		{
			error_value(#fmt["HOURS: expected dateTime but found %?.", operand])
		}
	}
}

fn minutes_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value(value.tm_min as i64)
		}
		_
		{
			error_value(#fmt["MINUTES: expected dateTime but found %?.", operand])
		}
	}
}

fn seconds_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			int_value(value.tm_sec as i64)
		}
		_
		{
			error_value(#fmt["SECONDS: expected dateTime but found %?.", operand])
		}
	}
}

// TODO: add timezone (this is supposed to return a xs:dayTimeDuration, see <http://www.w3.org/TR/xpath-datamodel/#types>)

fn tz_fn(operand: object) -> object
{
	alt operand
	{
		dateTime_value(value)
		{
			string_value(value.tm_zone, "")		// TODO: doubt this is correct
		}
		_
		{
			error_value(#fmt["SECONDS: expected dateTime but found %?.", operand])
		}
	}
}
