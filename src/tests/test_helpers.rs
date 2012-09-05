use io::WriterUtil;
use object::*;
use query::*;
use solution::*;
use sparql::*;
use store::*;

export check_bgp, check_strs, check_operands, check_triples, check_solution, check_solution_err;

fn check_strs(actual: ~str, expected: ~str) -> bool
{
	if actual != expected
	{
		io::stderr().write_line(fmt!("Found '%s', but expected '%s'", actual, expected));
		return false;
	}
	return true;
}

fn check_operands(actual: object, expected: object) -> bool
{
	if actual != expected
	{
		io::stderr().write_line("Found:");
		io::stderr().write_line(fmt!("   %?", actual));
		io::stderr().write_line("but expected:");
		io::stderr().write_line(fmt!("   %?", expected));
		return false;
	}
	return true;
}

fn check_bgp(groups: ~[solution], expected: solution) -> bool
{
	fn convert_bindings(group: solution) -> ~[~str]
	{
		do vec::map(group)
		|row|
		{
			let mut entries = ~[];
			for row.each |e| {vec::push(entries, fmt!("%s=%?", e.first(), e.second()))};
			let entries = std::sort::merge_sort(|x, y| *x <= *y, entries);
			str::connect(entries, ~", ")
		}
	}
	
	fn dump_bindings(actual: ~[~str])
	{
		io::stderr().write_line("Actual bindings:");
		for vec::eachi(actual)
		|i, bindings|
		{
			io::stderr().write_line(fmt!("   %?: %s", i, bindings));
		};
	}
	
	let mut actual = groups[0];
	for vec::each(vec::slice(groups, 1, groups.len())) 
	|group|
	{
		let store = create_store(~[], @std::map::str_hash());
		actual = join_solutions(&store, ~[~"*"], actual, group, false);
	}
	
	// Form this point forward we are dealing with [str] instead of [[binding]].
	let actual = convert_bindings(actual);
	let expected = convert_bindings(expected);
	
	let actual = std::sort::merge_sort(|x, y| *x <= *y, actual);
	let expected = std::sort::merge_sort(|x, y| *x <= *y, expected);
	
	if vec::len(actual) != vec::len(expected)
	{
		io::stderr().write_line(fmt!("Actual length is %?, but expected %?", vec::len(actual), vec::len(expected)));
		dump_bindings(actual);
		return false;
	}
	
	for vec::eachi(actual)
	|i, arow|
	{
		let erow = expected[i];
		
		if arow != erow
		{
			io::stderr().write_line(fmt!("Row #%? is %s, but expected %s", i, arow, erow));
			dump_bindings(actual);
			return false;
		}
	}
	
	return true;
}

fn check_triples(actual: ~[triple], expected: ~[triple]) -> bool
{
	fn dump_triples(actual: ~[triple])
	{
		io::stderr().write_line("Actual triples:");
		for vec::eachi(actual)
		|i, triple|
		{
			io::stderr().write_line(fmt!("   %?: %s", i, triple.to_str()));
		};
	}
	
	let actual = std::sort::merge_sort(|x, y| {x.subject <= y.subject}, actual);
	let expected = std::sort::merge_sort(|x, y| {x.subject <= y.subject}, expected);
	
	if vec::len(actual) != vec::len(expected)
	{
		io::stderr().write_line(fmt!("Actual length is %?, but expected %?", vec::len(actual), vec::len(expected)));
		dump_triples(actual);
		return false;
	}
	
	for vec::eachi(actual)
	|i, atriple|
	{
		let etriple = expected[i];
		
		if atriple.subject != etriple.subject
		{
			io::stderr().write_line(fmt!("Subject #%? is %?, but expected %?", i, atriple.subject, etriple.subject));
			dump_triples(actual);
			return false;
		}
		
		if atriple.predicate != etriple.predicate
		{
			io::stderr().write_line(fmt!("Predicate #%? is %?, but expected %?", i, atriple.predicate, etriple.predicate));
			dump_triples(actual);
			return false;
		}
		
		if atriple.object != etriple.object
		{
			io::stderr().write_line(fmt!("Object #%? is %s, but expected %s", i, atriple.object.to_str(), etriple.object.to_str()));
			dump_triples(actual);
			return false;
		}
	}
	
	return true;
}

fn check_solution(store: store, expr: ~str, expected: solution) -> bool
{
	info!("----------------------------------------------------");
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			match selector(&store)
			{
				result::Ok(actual) =>
				{
					// OK if they are both empty.
					if vec::is_empty(actual) && vec::is_empty(expected)
					{
						return true;
					}
					
					// Both sides should have the same number of rows.
					if vec::len(actual) != vec::len(expected)
					{
						print_failure(#fmt["Actual result had %? rows but expected %? rows.", 
							vec::len(actual), vec::len(expected)], actual, expected);
						return false;
					}
					
					// Actual should have only the expected values.
					for vec::eachi(actual)
					|i, row1|
					{
						let row2 = expected[i];
						if vec::len(row1) != vec::len(row2)
						{
							print_failure(#fmt["Row %? had size %? but expected %?.",
								i, vec::len(row1), vec::len(row2)], actual, expected);
							return false;
						}
						
						for row1.each
						|entry1|
						{
							let name1 = entry1.first();
							let value1 = entry1.second();
							match row2.search(name1)
							{
								option::Some(value2) =>
								{
									if value1 != value2
									{
										print_failure(#fmt["Row %? actual %s was %s but expected %s.",
											i, name1, value1.to_str(), value2.to_str()], actual, expected);
										return false;
									}
								}
								option::None =>
								{
									print_failure(#fmt["Row %? had unexpected ?%s.",
										i, name1], actual, expected);
									return false;
								}
							}
						};
					};
					
					return true;
				}
				result::Err(mesg) =>
				{
					io::stderr().write_line(fmt!("Eval error: %s", mesg));
					return false;
				}
			}
		}
		result::Err(mesg) =>
		{
			io::stderr().write_line(fmt!("Parse error: %s", mesg));
			return false;
		}
	}
}

fn check_solution_err(store: store, expr: ~str, expected: ~str) -> bool
{
	info!("----------------------------------------------------");
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			match selector(&store)
			{
				result::Ok(_) =>
				{
					io::stderr().write_line(fmt!("Expr evaluated but expected to find error '%s'.", expected));
					return false;
				}
				result::Err(mesg) =>
				{
					if str::contains(mesg, expected)
					{
						return true;
					}
					else
					{
						io::stderr().write_line(fmt!("Actual eval error was '%s' but expected to find '%s'.", mesg, expected));
						return false;
					}
				}
			}
		}
		result::Err(mesg) =>
		{
			if str::contains(mesg, expected)
			{
				return true;
			}
			else
			{
				io::stderr().write_line(fmt!("Actual parse error was '%s' but expected to find '%s'.", mesg, expected));
				return false;
			}
		}
	}
}

// ---- Private Functions -----------------------------------------------------
fn print_result(value: solution)
{
	for vec::eachi(value)
	|i, row|
	{
		let mut entries = ~[];
		for row.each |e| {vec::push(entries, fmt!("%s = %s", e.first(), e.second().to_str()))};
		io::stderr().write_line(fmt!("   %?: %s", i, str::connect(entries, ~", ")));
	};
}

fn print_failure(mesg: ~str, actual: solution, expected: solution)
{
	io::stderr().write_line(mesg);
	io::stderr().write_line("Actual:");
	print_result(actual);
	io::stderr().write_line("Expected:");
	print_result(expected);
}
