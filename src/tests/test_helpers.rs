import io;
import io::writer_util;
import query::*;

export bind_int, bind_str, bind_uri, check_bgp, check_strs, check_triples, check_solution, check_solution_err;

fn bind_int(name: str, value: int) -> binding
{
	{name: name, value: {value: #fmt["%?", value], kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""}}
}

fn bind_str(name: str, value: str) -> binding
{
	{name: name, value: {value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""}}
}

fn bind_uri(name: str, value: str) -> binding
{
	{name: name, value: {value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""}}
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

fn check_bgp(groups: [solution], expected: solution) -> bool
{
	fn convert_bindings(group: solution) -> [str]
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
		for vec::eachi(actual)
		{|i, bindings|
			io::stderr().write_line(#fmt["   %?: %s", i, bindings]);
		};
	}
	
	let mut actual = [];
	for vec::each(groups)
	{|group|
		actual = eval_bg_pair(actual, group);
	};
	
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
		for vec::eachi(actual)
		{|i, triple|
			io::stderr().write_line(#fmt["   %?: %s", i, triple.to_str()]);
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
		
		if atriple.object.value != etriple.object.value
		{
			io::stderr().write_line(#fmt["Object #%? value is %?, but expected %?", i, atriple.object.value, etriple.object.value]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.object.kind != etriple.object.kind
		{
			io::stderr().write_line(#fmt["Object #%? kind is %?, but expected %?", i, atriple.object.kind, etriple.object.kind]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.object.lang != etriple.object.lang
		{
			io::stderr().write_line(#fmt["Object #%? lang is %?, but expected %?", i, atriple.object.lang, etriple.object.lang]);
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
					// OK if they are both empty.
					if vec::is_empty(actual) && vec::is_empty(expected)
					{
						ret true;
					}
					
					// Both sides should have the same number of rows.
					if vec::len(actual) != vec::len(expected)
					{
						print_failure(#fmt["Actual result had %? rows but expected %? rows.", 
							vec::len(actual), vec::len(expected)], actual, expected);
						ret false;
					}
					
					// Both sides should have the same binding values.
					for vec::eachi(actual)
					{|i, row1|
						let row2 = expected[i];
						for vec::eachi(row1)
						{|j, binding1|
							let binding2 = row2[j];
							if binding1.name != binding2.name
							{
								print_failure(#fmt["Row %? actual name was %s but expected %s.", 
									i, binding1.name, binding2.name], actual, expected);
								ret false;
							}
							else if binding1.value.lang != binding2.value.lang
							{
								print_failure(#fmt["Row %? actual %s was %s but expected lang %s.", 
									i, binding1.name, binding1.value.to_str(), binding2.value.lang], actual, expected);
								ret false;
							}
							else if binding1.value.kind != binding2.value.kind
							{
								print_failure(#fmt["Row %? actual %s was %s but expected kind %s.", 
									i, binding1.name, binding1.value.to_str(), binding2.value.kind], actual, expected);
								ret false;
							}
							else if binding1.value.value != binding2.value.value
							{
								print_failure(#fmt["Row %? actual %s was %s but expected value %s.", 
									i, binding1.name, binding1.value.to_str(), binding2.value.value], actual, expected);
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
fn print_result(value: solution)
{
	for vec::eachi(value)
	{|i, row|
		let bindings = vec::map(row) {|b| #fmt["%s = %s", b.name, b.value.to_str()]};
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
