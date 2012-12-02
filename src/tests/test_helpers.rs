use io::WriterUtil;

pub fn check_strs(actual: ~str, expected: ~str) -> bool
{
	if actual != expected
	{
		io::stderr().write_line(fmt!("Found '%s', but expected '%s'", actual, expected));
		return false;
	}
	return true;
}

pub fn check_operands(actual: &Object, expected: &Object) -> bool
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

pub fn check_bgp(store: &Store, groups: &[Solution], expected: &Solution) -> bool
{
	fn dump_actual(store: &Store, solution: &Solution, rows: &[SolutionRow])
	{
		io::stderr().write_line("Actual bindings:");
		for vec::eachi(rows) |i, row|
		{
			io::stderr().write_line(fmt!("%?: %s   ", i, solution_row_to_str(store, solution, row)));
		};
	}
	
	let mut actual = copy groups[0];
	for vec::each(vec::slice(groups, 1, groups.len())) |group|
	{
		let store = Store(~[], &HashMap());
		actual = join_solutions(&store, &actual, group, false);
	}
	
	let actual_rows = std::sort::merge_sort(|x, y| {x <= y}, actual.rows);
	let expected_rows = std::sort::merge_sort(|x, y| {x <= y}, expected.rows);
	
	if actual_rows.len() != expected_rows.len()
	{
		io::stderr().write_line(fmt!("Actual length is %?, but expected %?", actual_rows.len(), expected_rows.len()));
		dump_actual(store, &actual, actual_rows);
		return false;
	}
	
	for vec::eachi(actual_rows) |i, arow|
	{
		let erow = copy expected_rows[i];
		
		if *arow != erow
		{
			io::stderr().write_line(fmt!("Row #%? is %s, but expected %s", i, solution_row_to_str(store, &actual, arow), solution_row_to_str(store, expected, &erow)));
			dump_actual(store, &actual, actual_rows);
			return false;
		}
	}
	
	return true;
}

pub fn check_triples(actual: &[Triple], expected: &[Triple]) -> bool
{
	fn dump_triples(actual: &[Triple])
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
	
	if actual.len() != expected.len()
	{
		io::stderr().write_line(fmt!("Actual length is %?, but expected %?", actual.len(), expected.len()));
		dump_triples(actual);
		return false;
	}
	
	for vec::eachi(actual)
	|i, atriple|
	{
		let etriple = copy expected[i];
		
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

pub fn check_eval(store: &Store, expr: ~str, expected: &Solution) -> bool
{
	info!("----------------------------------------------------");
	let expected = expected.sort();
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			match selector(store)
			{
				result::Ok(ref actual) =>
				{
					check_solution(actual, &expected)
				}
				result::Err(ref mesg) =>
				{
					io::stderr().write_line(fmt!("Eval error: %s", *mesg));
					return false;
				}
			}
		}
		result::Err(ref mesg) =>
		{
			io::stderr().write_line(fmt!("Parse error: %s", *mesg));
			return false;
		}
	}
}

pub fn check_solution(actual: &Solution, expected: &Solution) -> bool
{
	assert actual.bindings.len() == expected.bindings.len();
	assert actual.num_selected == expected.num_selected;
	
	let actual = actual.sort();
	
	// OK if they are both empty.
	if vec::is_empty(actual.rows) && vec::is_empty(expected.rows)
	{
		return true;
	}
	
	// Both sides should have the same number of rows.
	if actual.rows.len() != expected.rows.len()
	{
		print_failure(#fmt["Actual result had %? rows but expected %? rows.", 
			actual.rows.len(), expected.rows.len()], &actual, expected);
		return false;
	}
	
	// Actual should have only the expected values.
	for uint::range(0, actual.rows.len()) |i|
	{
		let row1 = actual.rows[i].slice(0, actual.num_selected);
		let row2 = expected.rows[i].slice(0, expected.num_selected);
		if row1.len() != row2.len()
		{
			print_failure(#fmt["Row %? had size %? but expected %?.",
				i, row1.len(), row2.len()], &actual, expected);
			return false;
		}
		
		for row1.eachi |i, entry1|
		{
			let name1 = copy actual.bindings[i];
			let value1 = *entry1;
			let value2 = row2[i];
			if value1 != value2
			{
				print_failure(#fmt["Row %? actual %s was %s but expected %s.",
					i, name1, value1.to_str(), value2.to_str()], &actual, expected);
				return false;
			}
		};
	};
	
	return true;
}

pub fn check_solution_err(store: &Store, expr: ~str, expected: ~str) -> bool
{
	info!("----------------------------------------------------");
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			match selector(store)
			{
				result::Ok(_) =>
				{
					io::stderr().write_line(fmt!("Expr evaluated but expected to find error '%s'.", expected));
					return false;
				}
				result::Err(ref mesg) =>
				{
					if str::contains(*mesg, expected)
					{
						return true;
					}
					else
					{
						io::stderr().write_line(fmt!("Actual eval error was '%s' but expected to find '%s'.", *mesg, expected));
						return false;
					}
				}
			}
		}
		result::Err(ref mesg) =>
		{
			if str::contains(*mesg, expected)
			{
				return true;
			}
			else
			{
				io::stderr().write_line(fmt!("Actual parse error was '%s' but expected to find '%s'.", *mesg, expected));
				return false;
			}
		}
	}
}

// ---- Private Functions -----------------------------------------------------
fn print_result(value: &Solution)
{
	for vec::eachi(value.rows)
	|i, row|
	{
		let mut entries = ~[];
		for row.eachi |i, e| {vec::push(&mut entries, fmt!("%s = %s", value.bindings[i], e.to_str()))};
		io::stderr().write_line(fmt!("   %?: %s", i, str::connect(entries, ~", ")));
	};
}

fn print_failure(mesg: ~str, actual: &Solution, expected: &Solution)
{
	io::stderr().write_line(mesg);
	io::stderr().write_line("Actual:");
	print_result(actual);
	io::stderr().write_line("Expected:");
	print_result(expected);
}
