import io;
import io::writer_util;
import query::*;

export ref_uri, ref_uri_none, ref_uri_str, str_ref_uri, check_algebra, check_strs, check_triples, check_solution, check_solution_err;

fn ref_uri(r: str, u: str) -> [option<object>]
{
	[option::some({value: r, kind: "xsd:anyURI", lang: ""}), option::some({value: u, kind: "xsd:anyURI", lang: ""})]
}

fn ref_uri_none(r: str, u: str) -> [option<object>]
{
	[option::some({value: r, kind: "xsd:anyURI", lang: ""}), option::some({value: u, kind: "xsd:anyURI", lang: ""}), option::none]
}

fn ref_uri_str(r: str, u: str, s: str) -> [option<object>]
{
	[option::some({value: r, kind: "xsd:anyURI", lang: ""}), option::some({value: u, kind: "xsd:anyURI", lang: ""}), option::some({value: s, kind: "xsd:string", lang: ""})]
}

fn str_ref_uri(s: str, r: str, u: str) -> [option<object>]
{
	[option::some({value: s, kind: "xsd:string", lang: ""}), option::some({value: r, kind: "xsd:anyURI", lang: ""}), option::some({value: u, kind: "xsd:anyURI", lang: ""})]
}

fn check_strs(actual: str, expected: str) -> bool
{
	if actual != expected
	{
		io::stderr().write_line(#fmt["Found '%s', but expected '%s'", actual, expected]);
		ret false;
	}
	ret true;
}

fn check_algebra(store: store, groups: solution_groups, expected: solution_group) -> bool
{
	fn convert_bindings(group: solution_group) -> [str]
	{
		vec::map(group)
		{|row|
			let row = std::sort::merge_sort({|x, y| x.name <= y.name}, row);
			let entries = vec::map(row, {|b| #fmt["%s=%?", b.name, b.value]});
			str::connect(entries, ", ")
		}
	}
	
	fn dump_bindings(actual: [str])
	{
		io::stderr().write_line("Actual bindings:");
		for vec::each(actual)
		{|bindings|
			io::stderr().write_line(#fmt["   %s", bindings]);
		};
	}
	
	let actual = eval_bgp(store, groups);
	
	// Form this point forward we are dealing with [str] instead of [[binding]].
	let actual = convert_bindings(actual);
	let expected = convert_bindings(expected);
	
	let actual = std::sort::merge_sort({|x, y| x <= y}, actual);
	let expected = std::sort::merge_sort({|x, y| x <= y}, expected);
	
	if vec::len(actual) != vec::len(expected)
	{
		io::stderr().write_line(#fmt["Actual length is %?, but expected %?", vec::len(actual), vec::len(expected)]);
		dump_bindings(actual);
		ret false;
	}
	
	for vec::eachi(actual)
	{|i, arow|
		let erow = expected[i];
		
		if arow != erow
		{
			io::stderr().write_line(#fmt["Row #%? is %s, but expected %s", i, arow, erow]);
			dump_bindings(actual);
			ret false;
		}
	}
	
	ret true;
}

fn check_triples(actual: [triple], expected: [triple]) -> bool
{
	fn dump_triples(actual: [triple])
	{
		io::stderr().write_line("Actual triples:");
		for vec::each(actual)
		{|triple|
			io::stderr().write_line(#fmt["   %s", triple.to_str()]);
		};
	}
	
	let actual = std::sort::merge_sort({|x, y| x.subject <= y.subject}, actual);
	let expected = std::sort::merge_sort({|x, y| x.subject <= y.subject}, expected);
	
	if vec::len(actual) != vec::len(expected)
	{
		io::stderr().write_line(#fmt["Actual length is %?, but expected %?", vec::len(actual), vec::len(expected)]);
		dump_triples(actual);
		ret false;
	}
	
	for vec::eachi(actual)
	{|i, atriple|
		let etriple = expected[i];
		
		if atriple.subject != etriple.subject
		{
			io::stderr().write_line(#fmt["Subject #%? is %?, but expected %?", i, atriple.subject, etriple.subject]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.predicate != etriple.predicate
		{
			io::stderr().write_line(#fmt["Predicate #%? is %?, but expected %?", i, atriple.predicate, etriple.predicate]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.object != etriple.object
		{
			io::stderr().write_line(#fmt["Object #%? is %?, but expected %?", i, atriple.object.to_str(), etriple.object.to_str()]);
			dump_triples(actual);
			ret false;
		}
	}
	
	ret true;
}

fn check_solution(store: store, expr: str, expected: solution) -> bool
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

fn check_solution_err(store: store, expr: str, expected: str) -> bool
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
