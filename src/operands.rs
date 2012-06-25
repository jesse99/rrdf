// Values used within SPARQL FILTER expressions. See 17.2 and related.

// Note that the error conditions do not always result in an error:
// 1) Functions like COALESCE accept unbound variables.
// 2) Boolean functions normally want effective boolean values which are false for invalid values.
// 3) Functions like op_and do not always propagate errors.
enum operand
{
	// literals
	bool_value(bool),
	int_value(i64),			// xsd:decimal (and derived types)
	float_value(f64),			// xsd:float or xsd:double
	dateTime_value(tm),	// xsd:dateTime
	string_value(str, str),	// value + lang
	typed_value(str, str),	// value + type (aka simple literal)
	
	// other rdf terms
	iri_value(str),
	blank_value(str),
	
	// error conditions
	unbound_value(str),	// binding name
	invalid_value(str),		// err mesg (for literal with invalid representation)
	error_value(str)			// err mesg
}

fn get_operand(row: solution_row, name: str) -> operand
{
	alt row.search(name)
	{
		// TODO: return invalid_value for bad bool and numeric
		option::some(value)
		{
			error_value(#fmt["get_operand ?%s is was bound to an unsupported type: %s.", name, value.kind])
		}
		option::none
		{
			unbound_value(name)
		}
	}
}

// 17.2.2
fn get_ebv(operand: operand) -> result::result<bool, str>
{
	alt operand
	{
		invalid_value(_err)
		{
			result::ok(false)
		}
		bool_value(value)
		{
			result::ok(value)
		}
		string_value(value, _) | typed_value(value, _)
		{
			result::ok(str::is_not_empty(value))
		}
		int_value(value)
		{
			result::ok(value != 0i64)
		}
		float_value(value)
		{
			result::ok(!f64::is_NaN(value) && value != 0f64)
		}
		unbound_value(name)
		{
			result::err(#fmt["?%s is not bound.", name])
		}
		error_value(err)
		{
			result::err(err)
		}
		_
		{
			result::err(#fmt["%? cannot be converted into an effective boolean value.", operand])
		}
	}
}

fn type_error(fname: str, operand: operand, expected: str) -> str
{
	alt operand
	{
		unbound_value(name)
		{
			#fmt["%s: ?%s was not bound.", fname, name]
		}
		invalid_value(err)
		{
			#fmt["%s: %s", fname, err]
		}
		error_value(err)
		{
			#fmt["%s: %s", fname, err]
		}
		_
		{
			#fmt["%s: expected %s value but found %?.", fname, expected, operand]
		}
	}
}

