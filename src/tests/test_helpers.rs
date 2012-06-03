import io;
import io::writer_util;

export check_result;

fn check_result(triples: [triple], expr: str, expected: query::result) -> bool
{
	#info["----------------------------------------------------"];
	let actual = sparql::query(triples, expr);
	
	// Both sides should have the same number of bindings.
	if vec::len(actual.names) != vec::len(expected.names)
	{
		print_failure(#fmt["Actual result had %? bindings but expected %? bindings.", 
			vec::len(actual.names), vec::len(expected.names)], actual, expected);
		ret false;
	}
	
	// Both sides should have the same binding names.
	let names1 = str::connect(actual.names, " ");
	let names2 = str::connect(expected.names, " ");
	if names1 != names2
	{
		print_failure(#fmt["Actual binding names are '%s' but expected '%s'.", 
			names1, names2], actual, expected);
		ret false;
	}
	
	// Both sides should have the same number of rows.
	if vec::len(actual.rows) != vec::len(expected.rows)
	{
		print_failure(#fmt["Actual result had %? results but expected %? results.", 
			vec::len(actual.rows), vec::len(expected.rows)], actual, expected);
		ret false;
	}
	
	// Both sides should have the same binding values.
	for vec::eachi(actual.rows)
	{|i, row1|
		let row2 = expected.rows[i];
		for vec::eachi(row1)
		{|j, value1|
			let value2 = row2[j];
			if value1 != value2
			{
				print_failure(#fmt["Row %? actual %s value was %s but expected %s.", 
					i, actual.names[j], oo_to_str(value1), oo_to_str(value2)], actual, expected);
				ret false;
			}
		};
	};
	
	ret true;
}

// ---- Private Functions -----------------------------------------------------
fn oo_to_str(value: option<object>) -> str
{
	alt value
	{
		some(v)
		{
			v.to_str()
		}
		none
		{
			"<none>"
		}
	}
}

fn print_result(value: query::result)
{
	for vec::eachi(value.rows)
	{|i, row|
		let pairs = vec::zip(value.names, vec::map(row) {|r| oo_to_str(r)});
		let bindings = vec::map(pairs) {|p| #fmt["%s = %s", tuple::first(p), tuple::second(p)]};
		io::stderr().write_line(#fmt["   %?: %s", i, str::connect(bindings, ", ")]);
	}
}

fn print_failure(mesg: str, actual: query::result, expected: query::result)
{
	io::stderr().write_line(mesg);
	io::stderr().write_line("Actual:");
	print_result(actual);
	io::stderr().write_line("Expected:");
	print_result(expected);
}
