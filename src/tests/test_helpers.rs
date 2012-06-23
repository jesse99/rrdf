import io;
import io::writer_util;
import query::*;

export add_solutions, check_bgp, check_strs, check_triples, check_solution, check_solution_err;

impl add_solutions for solution_row
{
	fn add_int(name: str, value: int) -> solution_row
	{
		self.insert(name, {value: #fmt["%?", value], kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""});
		self
	}
	
	fn add_str(name: str, value: str) -> solution_row
	{
		self.insert(name, {value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""});
		self
	}
	
	fn add_uri(name: str, value: str) -> solution_row
	{
		self.insert(name, {value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""});
		self
	}
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
			let mut entries = [];
			for row.each {|name, value| vec::push(entries, #fmt["%s=%?", name, value])};
			let entries = std::sort::merge_sort({|x, y| x <= y}, entries);
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
					
					// Actual should have all the expected values.
					for vec::eachi(actual)
					{|i, row1|
						let row2 = expected[i];
						if row1.size() < row2.size()
						{
							print_failure(#fmt["Row %? had size %? but expected at least%?.", 
								i, row1.size(), row2.size()], actual, expected);
							ret false;
						}
						
						for row1.each
						{|name1, value1|
							alt row2.find(name1)
							{
								option::some(value2)
								{
									if value1.lang != value2.lang
									{
										print_failure(#fmt["Row %? actual %s was %s but expected lang %s.", 
											i, name1, value1.to_str(), value2.lang], actual, expected);
										ret false;
									}
									else if value1.kind != value2.kind
									{
										print_failure(#fmt["Row %? actual %s was %s but expected kind %s.", 
											i, name1, value1.to_str(), value2.kind], actual, expected);
										ret false;
									}
									else if value1.value != value2.value
									{
										print_failure(#fmt["Row %? actual %s was %s but expected value %s.", 
											i, name1, value1.to_str(), value2.value], actual, expected);
										ret false;
									}
								}
								option::none
								{
									// Actual can have additional bindings.
								}
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
		let mut entries = [];
		for row.each {|name, value| vec::push(entries, #fmt["%s = %s", name, value.to_str()])};
		io::stderr().write_line(#fmt["   %?: %s", i, str::connect(entries, ", ")]);
	};
}

fn print_failure(mesg: str, actual: solution, expected: solution)
{
	io::stderr().write_line(mesg);
	io::stderr().write_line("Actual:");
	print_result(actual);
	io::stderr().write_line("Expected:");
	print_result(expected);
}
