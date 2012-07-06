[Resource Description Language](http://www.w3.org/RDF/) (RDF) library implemented with the Rust programming language.

The library contains an implementation of a triple store and a large subset of the [SPARQL 1.1](http://www.w3.org/TR/2012/WD-sparql11-query-20120105/) query language. There is not currently any support for serialization via XML or [Turtle](http://www.w3.org/TR/2011/WD-turtle-20110809/).

SPARQL support includes:
* Nearly all the operators and functions.
* User defined extension functions.
* Pattern groups.
* Optional, bind, filter, and order by clauses.
* Distinct and limit modifiers.
* Namespaces.

SPARQL support does not include:
* Construct, describe, and ask queries.
* Union patterns
* Offset and having clauses.
* As in select.
* Aggregate expressions.
* Path operators (i.e. \^, \/, and \|).
* The regex and replace functions.
* Hash functions.
* Nil literals.

Here is a usage example:

    import io::writer_util;
    import rrdf::*;
    
    // Creates a triple store and adds monsters to it.
    fn monsters() -> store
    {
        // Namespaces allow subjects and predicates to be added to the 
        // store using a prefixed name instead of a full URL.
        let namespaces = [{prefix: "game", path: "http://game/ns#"}];
        
        // Vector of function name and function pointer tuples. These
        // represent user defined functions that may be called from
        // within SPARQL queries.
        let extensions = ~[];
        
        // Create an empty triple store.
        let store = create_store(namespaces, extensions);
        
        // Start adding statements to the store. Individual triples may be added,
        // containers, aggregates, reified statements, and predicates/objects
        // associated with a subject (which is what we use here).
        store.add("game:snake", ~[
            ("game:name", string_value("King Snake", "")),    // "" is for an optional language
            ("game:min_level", int_value(1)),
            ("game:max_level", int_value(5)),
            ("game:weight", int_value(4)),                // relative probability
            ("game:habitat", string_value("|land|water|", "")),
        ]);
        
        store.add("game:bear", ~[
            ("game:name", string_value("Grizzly Bear", "")),
            ("game:min_level", int_value(3)),
            ("game:max_level", int_value(6)),
            ("game:weight", int_value(3)),
            ("game:habitat", string_value("|land|", "")),
        ]);
        
        store.add("game:naga", ~[
            ("game:name", string_value("Naga Warrior", "")),
            ("game:min_level", int_value(7)),
            ("game:max_level", int_value(15)),
            ("game:weight", int_value(2)),
            ("game:habitat", string_value("|land|water|", "")),
        ]);
        
        store.add("game:shark", ~[
            ("game:name", string_value("Hammerhead Shark", "")),
            ("game:min_level", int_value(5)),
            ("game:max_level", int_value(21)),
            ("game:weight", int_value(1)),
            ("game:habitat", string_value("|water|", "")),
        ]);
        
        store.add("game:mummy", ~[
            ("game:name", string_value("Mummy", "")),
            ("game:min_level", int_value(10)),
            ("game:max_level", int_value(20)),
            ("game:weight", int_value(2)),
            ("game:habitat", string_value("|land|", "")),
        ]);
        
        store.add("game:lich", ~[
            ("game:name", string_value("Lich", "")),
            ("game:min_level", int_value(15)),
            ("game:max_level", int_value(30)),
            ("game:weight", int_value(3)),
            ("game:habitat", string_value("|land|", "")),
            ("game:announce", string_value("You feel a chill.", "")),    
        ]);
        
        store.add("game:necromancer", ~[
            ("game:name", string_value("Necromancer", "")),
            ("game:min_level", int_value(20)),
            ("game:max_level", int_value(30)),
            ("game:weight", int_value(2)),
            ("game:habitat", string_value("|land|", "")),
        ]);
        
        ret store;
    }
    
    fn query_monsters()
    {
        // Return the names and weights for all land monsters allowed on level 20.
        // If the monster has an announcement then return that as well
        let expr = "PREFIX game: <http://game/ns#>
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
        alt compile(expr)
        {
            result::ok(selector)
            {
                // Run the query function against the store. This will either return
                // a row for each monster that matched the query or an eval error.
                alt selector(store)
                {
                    result::ok(solution)
                    {
                        // This will print:
                        // 0: [(name, "Lich"), (weight, 3), (announcement, "You feel a chill.")]
                        // 1: [(name, "Mummy"), (weight, 2)]
                        // 2: [(name, "Necromancer"), (weight, 2)]
                        for vec::eachi(solution)
                        |i, row|
                        {
                            io::println(#fmt["%?: %s", i, row.to_str()]);
                        };
                    }
                    result::err(err)
                    {
                        io::stderr().write_line(#fmt["Eval error: %s", err]);
                    }
                }
            }
            result::err(err)
            {
                io::stderr().write_line(#fmt["Parse error: expected %s", err]);
            }
        }
    }
    
