// The sparql parser operates by building a sequence of matcher functions and
// then creating a selector function using the select function.
import std::map::hashmap;

type solution = {names: [str], rows: [[core::option::option<object>]]};	// len(names) == len(rows[x])

type binding = {name: str, value: option<object>};
type match = core::either::either<[binding], bool>;			// match succeeded if bindings or true
type matcher = fn@ (triple) -> match;

type selector = fn@ ([triple]) -> solution;

fn variable_subject(name: str) -> matcher
{
	{|triplet: triple|
		let s = triplet.subject;
		let b = {name: name, value: some(reference(s))};
		core::either::left([b])
	}
}

fn variable_property(name: str) -> matcher
{
	{|triplet: triple|
		let b = {name: name, value: some(anyURI(triplet.property))};
		core::either::left([b])
	}
}

fn variable_object(name: str) -> matcher
{
	{|triplet: triple|
		let b = {name: name, value: some(triplet.object)};
		core::either::left([b])
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
									row[index] = binding.value;
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
		
		{names: names, rows: rows}
	}
}
