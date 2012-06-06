// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;

#[doc = "Names appear in the same order as the variables in the SELECT clause.

Each returned row will name (len(names) columns."]
type solution = {names: [str], rows: [[option<object>]]};

type binding = {name: str, value: option<object>};
type match = either::either<[binding], bool>;					// match succeeded if bindings or true
type matcher = fn@ (triple) -> match;

type selector = fn@ ([triple]) -> result::result<solution, str>;	// returns a solution or an error

enum pattern
{
	variable(str),
	string_literal(str)
}

fn match_subject(pattern: pattern) -> matcher
{
	{|triplet: triple|
		let s = triplet.subject;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(reference(s))};
				core::either::left([b])
			}
			string_literal(text)
			{
				core::either::right(text == s.to_str())
			}
		}
	}
}

fn match_property(pattern: pattern) -> matcher
{
	{|triplet: triple|
		let p = triplet.property;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(anyURI(p))};
				core::either::left([b])
			}
			string_literal(text)
			{
				core::either::right(text == p)
			}
		}
	}
}

fn match_object(pattern: pattern) -> matcher
{
	{|triplet: triple|
		let o = triplet.object;
		
		alt pattern
		{
			variable(name)
			{
				let b = {name: name, value: some(triplet.object)};
				core::either::left([b])
			}
			string_literal(text)
			{
				alt o
				{
					string(s)
					{
						core::either::right(text == s)
					}
					_
					{
						core::either::right(false)
					}
				}
			}
		}
	}
}

// Returns the named bindings. Binding values for names not 
// returned by the matchers are set to none.
fn select(names: [str], matchers: [matcher]) -> selector
{
	{|triples: [triple]|
		let mut rows: [[option<object>]] = [];
		
		for vec::each(triples)
		{|triple|
			let row: [mut option<object>] = vec::to_mut(vec::from_elem(vec::len(names), option::none));
			let mut matched = true;
			
			for vec::each(matchers)
			{|matcher|
				alt matcher(triple)
				{
					core::either::left(bindings)
					{
						for vec::each(bindings)
						{|binding|
							alt vec::position_elem(names, binding.name)
							{
								option::some(index)
								{
									if row[index] == option::none
									{
										row[index] = binding.value;
									}
									else
									{
										// Spec isn't clear what the semantics of this should be, but it seems
										// likely to be, at best, confusing and normally a bug so we'll call it
										// an error for now.
										ret result::err(#fmt["Binding %s was set more than once.", binding.name]);
									}
								}
								option::none
								{
									// Matcher created a binding, but it's not one the user wanted returned
									// (though it could be used by other matchers).
								}
							}
						};
					}
					core::either::right(true)
					{
						// We matched the triple so we can keep going, but this
						// matcher doesn't have any bindings so there is nothing
						// to return for it.
					}
					core::either::right(false)
					{
						matched = false;
						break;
					}
				}
			};
			
			if matched
			{
				vec::push(rows, vec::from_mut(row));	// TODO: may be able to speed up the vector conversions using unsafe functions
			}
		};
		
		result::ok({names: names, rows: rows})
	}
}
