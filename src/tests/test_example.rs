/// This is the example code we drop into the readme.
use io::WriterUtil;
use object::*;
use sparql::*;
use solution::*;
use store::*;
use test_helpers::*;

// Creates a triple store and adds monsters to it.
fn monsters() -> Store
{
	// Namespaces allow subjects and predicates to be added to the 
	// store using a prefixed name instead of a full URL.
	let namespaces = ~[{prefix: ~"game", path: ~"http://game/ns#"}];
	
	// Vector of function name and function pointer tuples. These
	// represent user defined functions that may be called from
	// within SPARQL queries.
	let extensions = std::map::HashMap();
	
	// Create an empty triple store.
	let store = Store(namespaces, &extensions);
	
	// Start adding statements to the store. Individual triples may be added,
	// containers, aggregates, reified statements, and predicates/objects
	// associated with a subject (which is what we use here).
	store.add(~"game:snake", ~[
		(~"game:name", StringValue(~"King Snake", ~"")),	// "" is for an optional language
		(~"game:min_level", IntValue(1)),
		(~"game:max_level", IntValue(5)),
		(~"game:weight", IntValue(4)),				// relative probability
		(~"game:habitat", StringValue(~"|land|water|", ~"")),
	]);
	
	store.add(~"game:bear", ~[
		(~"game:name", StringValue(~"Grizzly Bear", ~"")),
		(~"game:min_level", IntValue(3)),
		(~"game:max_level", IntValue(6)),
		(~"game:weight", IntValue(3)),
		(~"game:habitat", StringValue(~"|land|", ~"")),
	]);
	
	store.add(~"game:naga", ~[
		(~"game:name", StringValue(~"Naga Warrior", ~"")),
		(~"game:min_level", IntValue(7)),
		(~"game:max_level", IntValue(15)),
		(~"game:weight", IntValue(2)),
		(~"game:habitat", StringValue(~"|land|water|", ~"")),
	]);
	
	store.add(~"game:shark", ~[
		(~"game:name", StringValue(~"Hammerhead Shark", ~"")),
		(~"game:min_level", IntValue(5)),
		(~"game:max_level", IntValue(21)),
		(~"game:weight", IntValue(1)),
		(~"game:habitat", StringValue(~"|water|", ~"")),
	]);
	
	store.add(~"game:mummy", ~[
		(~"game:name", StringValue(~"Mummy", ~"")),
		(~"game:min_level", IntValue(10)),
		(~"game:max_level", IntValue(20)),
		(~"game:weight", IntValue(2)),
		(~"game:habitat", StringValue(~"|land|", ~"")),
	]);
	
	store.add(~"game:lich", ~[
		(~"game:name", StringValue(~"Lich", ~"")),
		(~"game:min_level", IntValue(15)),
		(~"game:max_level", IntValue(30)),
		(~"game:weight", IntValue(3)),
		(~"game:habitat", StringValue(~"|land|", ~"")),
		(~"game:announce", StringValue(~"You feel a chill.", ~"")),	
	]);
	
	store.add(~"game:necromancer", ~[
		(~"game:name", StringValue(~"Necromancer", ~"")),
		(~"game:min_level", IntValue(20)),
		(~"game:max_level", IntValue(30)),
		(~"game:weight", IntValue(2)),
		(~"game:habitat", StringValue(~"|land|", ~"")),
	]);
	
	return store;
}

#[test]
fn query_monsters()
{
	// Return the names and weights for all land monsters allowed on level 20.
	// If the monster has an announcement then return that as well
	let expr = ~"PREFIX game: <http://game/ns#>
		SELECT
			?name ?weight ?announcement
		WHERE
		{
			?subject game:name ?name .
			?subject game:weight ?weight .
			?subject game:min_level ?min .
			?subject game:max_level ?max .
			?subject game:habitat ?habitat .
			OPTIONAL
			{
				?subject game:announce ?announcement
			}
			FILTER (CONTAINS(?habitat, \"|land|\") && ?min <= 20 && 20 <= ?max)
		} ORDER BY ?name";
	
	// Parse the query expression and return a result with either a function
	// that will run the query against a store or parse error.
	let store = monsters();
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			// Run the query function against the store. This will either return
			// a row for each monster that matched the query or an eval error.
			match selector(&store)
			{
				result::Ok(ref solution) =>
				{
					// This will print:
					// 0: [(name, "Lich"), (weight, 3), (announcement, "You feel a chill.")]
					// 1: [(name, "Mummy"), (weight, 2)]
					// 2: [(name, "Necromancer"), (weight, 2)]
					for vec::eachi(solution.rows)
					|i, row|
					{
						io::println(fmt!("%?: %s", i, row.to_str()));
					};
				}
				result::Err(ref err) =>
				{
					io::stderr().write_line(fmt!("Eval error: %s", *err));
				}
			}
		}
		result::Err(ref err) =>
		{
			io::stderr().write_line(fmt!("Parse error: expected %s", *err));
		}
	}
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Lich", ~"")), (~"weight", IntValue(3)), (~"announcement", StringValue(~"You feel a chill.", ~""))],
		~[(~"name", StringValue(~"Mummy", ~"")), (~"weight", IntValue(2))],
		~[(~"name", StringValue(~"Necromancer", ~"")), (~"weight", IntValue(2))],
	]};
	assert check_solution(&store, expr, &expected);
}

