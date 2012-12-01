//! SPARQL functions. Clients will not ordinarily use this.

pub pure fn now_fn(context: &QueryContext, args: ~[@Object]) -> Object
{
	if vec::len(args) == 0u
	{
		DateTimeValue(copy context.timestamp)
	}
	else
	{
		ErrorValue(fmt!("NOW accepts 0 arguments but was called with %? arguments.", vec::len(args)))
	}
}

pub pure fn year_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue((1900i32 + value.tm_year) as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("YEAR: expected dateTime but found %?.", *operand))
		}
	}
}

pub pure fn month_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue((1i32 + value.tm_mon) as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("MONTH: expected dateTime but found %?.", *operand))
		}
	}
}

pub pure fn day_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue(value.tm_mday as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("DAY: expected dateTime but found %?.", *operand))
		}
	}
}

pub pure fn hours_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue(value.tm_hour as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("HOURS: expected dateTime but found %?.", *operand))
		}
	}
}

pub pure fn minutes_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue(value.tm_min as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("MINUTES: expected dateTime but found %?.", *operand))
		}
	}
}

pub pure fn seconds_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			IntValue(value.tm_sec as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("SECONDS: expected dateTime but found %?.", *operand))
		}
	}
}

// TODO: add timezone (this is supposed to return a xs:dayTimeDuration, see <http://www.w3.org/TR/xpath-datamodel/#types>)

pub pure fn tz_fn(operand: &Object) -> Object
{
	match *operand
	{
		DateTimeValue(ref value) =>
		{
			StringValue(copy value.tm_zone, ~"")		// TODO: doubt this is correct
		}
		_ =>
		{
			ErrorValue(fmt!("SECONDS: expected dateTime but found %?.", *operand))
		}
	}
}
