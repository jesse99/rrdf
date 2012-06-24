// Values used within SPARQL FILTER expressions. See 17.2 and related.
//import std::map::hashmap;
//import result::extensions;
//import std::time::tm;
//import sparql::*;

//export join_solutions, eval, pattern;

enum operand
{
	bool_value(bool),
	int_value(i64),			// xsd:integer (and derived types)
	float_value(f64),			// xsd:decimal, xsd:float, or xsd:double (and derived types)
	dateTime_value(tm),	// xsd:dateTime
	
	string_value(str, str),	// value + lang
	typed_value(str, str),	// value + type
	iri_value(str),
	blank_value(str),
	unbound_value(str),	// name
	
	invalid_value(str),		// bool or numeric with invalid representation
	error_value(str)			// we have to propagate errors because some operators special case them (e.g. ||)
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
		invalid_value(_value)
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
