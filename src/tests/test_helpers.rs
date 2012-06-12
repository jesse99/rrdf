import io;
import io::writer_util;

export ref_uri, ref_uri_none, ref_uri_str, str_ref_uri, check_ok, check_err;

fn ref_uri(r: str, u: str) -> [option<object>]
{
	[option::some(reference(r)), option::some(typed_literal(u, "xsd:anyURI"))]
}

fn ref_uri_none(r: str, u: str) -> [option<object>]
{
	[option::some(reference(r)), option::some(typed_literal(u, "xsd:anyURI")), option::none]
}

fn ref_uri_str(r: str, u: str, s: str) -> [option<object>]
{
	[option::some(reference(r)), option::some(typed_literal(u, "xsd:anyURI")), option::some(plain_literal(s, ""))]
}

fn str_ref_uri(s: str, r: str, u: str) -> [option<object>]
{
	[option::some(plain_literal(s, "")), option::some(reference(r)), option::some(typed_literal(u, "xsd:anyURI"))]
}

fn check_ok(store: store, expr: str, expected: solution) -> bool
{
	#info["----------------------------------------------------"];
	alt compile(expr)
	{
		result::ok(selector)
		{
			alt selector(store)
			{
				result::ok(actual)
				{
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
				result::err(mesg)
				{
					io::stderr().write_line(#fmt["Eval error: %s", mesg]);
					ret false;
				}
			}
		}
		result::err(mesg)
		{
			io::stderr().write_line(#fmt["Parse error: %s", mesg]);
			ret false;
		}
	}
}

fn check_err(store: store, expr: str, expected: str) -> bool
{
	#info["----------------------------------------------------"];
	alt compile(expr)
	{
		result::ok(selector)
		{
			alt selector(store)
			{
				result::ok(actual)
				{
					io::stderr().write_line(#fmt["Expr evaluated but expected to find error '%s'.", expected]);
					ret false;
				}
				result::err(mesg)
				{
					if str::contains(mesg, expected)
					{
						ret true;
					}
					else
					{
						io::stderr().write_line(#fmt["Actual eval error was '%s' but expected to find '%s'.", mesg, expected]);
						ret false;
					}
				}
			}
		}
		result::err(mesg)
		{
			if str::contains(mesg, expected)
			{
				ret true;
			}
			else
			{
				io::stderr().write_line(#fmt["Actual parse error was '%s' but expected to find '%s'.", mesg, expected]);
				ret false;
			}
		}
	}
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

fn print_result(value: solution)
{
	for vec::eachi(value.rows)
	{|i, row|
		let pairs = vec::zip(value.names, vec::map(row) {|r| oo_to_str(r)});
		let bindings = vec::map(pairs) {|p| #fmt["%s = %s", tuple::first(p), tuple::second(p)]};
		io::stderr().write_line(#fmt["   %?: %s", i, str::connect(bindings, ", ")]);
	}
}

fn print_failure(mesg: str, actual: solution, expected: solution)
{
	io::stderr().write_line(mesg);
	io::stderr().write_line("Actual:");
	print_result(actual);
	io::stderr().write_line("Expected:");
	print_result(expected);
}
