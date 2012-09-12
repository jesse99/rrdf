use std::map::*;
use object::*;
use solution::*;
use store::*;
use test_helpers::*;

fn got(s: ~str) -> ~str
{
	~"http://awoiaf.westeros.org/index.php/" + s
}

fn v(s: ~str) -> ~str
{
	~"http://www.w3.org/2006/vcard/ns#" + s
}

fn wiki(s: ~str) -> ~str
{
	~"http://en.wikipedia.org/wiki/" + s
}

#[test]
fn trivial()
{
	let expr = ~"SELECT ?s ?p ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"fn"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"nickname"))), (~"o", StringValue(~"Ned", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn out_of_order()
{
	let expr = ~"SELECT ?o ?s ?p WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"o", StringValue(~"Eddard Stark", ~"")), (~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"fn")))],
		~[(~"o", StringValue(~"Ned", ~"")), (~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"nickname")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_names()
{
	let expr = ~"SELECT ?subject ?p ?obj WHERE {?subject ?p ?obj}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"subject", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"fn"))), (~"obj", StringValue(~"Eddard Stark", ~""))],
		~[(~"subject", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"nickname"))), (~"obj", StringValue(~"Ned", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn keyword_case()
{
	let expr = ~"SeLecT ?s ?p ?o where {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"fn"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"nickname"))), (~"o", StringValue(~"Ned", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn duplicate_select_variables()
{
	let expr = ~"SELECT ?s ?s ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(store, expr, ~"Select clause has duplicates: s");
}

#[test]
fn duplicate_where_variables()
{
	let expr = ~"SELECT ?s ?p ?o WHERE {?s ?s ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(store, expr, ~"Binding s was set more than once.");
}

#[test]
fn unbound_variable()
{
	let expr = ~"SELECT ?s ?p ?z WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"fn")))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"p", IriValue(v(~"nickname")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn no_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p \"Peter Pan\"}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn comment()
{
	let expr = ~"SELECT ?s ?p #your comment here
	WHERE {	# yet another comment
		?s ?p \"Peter Pan\"
	}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn simple_path()
{
	let expr = ~"SELECT ?org
	WHERE {
		<http://awoiaf.westeros.org/index.php/Eddard_Stark> <http://www.w3.org/2006/vcard/ns#org> ?z .
		?z <http://www.w3.org/2006/vcard/ns#organisation-name> ?org
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"org", StringValue(~"Small Council", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn unmatched_path()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?subject
	WHERE
	{
		?subject wiki:phylum \"chordata\" .
		?subject wiki:class \"arachnid\"
	}";
	
	let store = Store(~[{prefix: ~"wiki", path: ~"http://en.wikipedia.org/wiki/"}], &std::map::box_str_hash());
	store.add(~"wiki:giraffe", ~[
		(~"wiki:phylum", StringValue(~"chordata", ~"")),
		(~"wiki:class", StringValue(~"mammalia", ~"")),
	]);
	
	let expected = Solution {namespaces: ~[], rows: ~[]};
	assert check_solution(store, expr, expected);
}

#[test]
fn unmatched_path2()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?subject
	WHERE
	{
		?subject wiki:phylum \"motie\" .
		?subject wiki:class \"mammalia\"
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[]};
	assert check_solution(store, expr, expected);
}

#[test]
fn unmatched_path3()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?subject ?family
	WHERE
	{
		?subject wiki:phylum \"arthropoda\" .
		?subject wiki:class \"mammalia\" .
		?subject wiki:family ?family
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[]};
	assert check_solution(store, expr, expected);
}

#[test]
fn select_all()
{
	let expr = ~"SELECT *
	WHERE {
		<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?o
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"p", IriValue(v(~"fn"))), (~"o", StringValue(~"Sandor Clegane", ~""))],
		~[(~"p", IriValue(v(~"nickname"))), (~"o", StringValue(~"The Hound", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn prefixes()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?org
	WHERE {
		got:Eddard_Stark v:org ?z .
		?z v:organisation-name ?org
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"org", StringValue(~"Small Council", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn options1()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title ?pet
	WHERE {
		?s v:fn ?name .
		OPTIONAL {
			?s v:honorific-prefix ?title
		}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Eddard Stark", ~"")), (~"title", StringValue(~"Lord", ~""))],
		~[(~"name", StringValue(~"Jon Snow", ~""))],
		~[(~"name", StringValue(~"Sandor Clegane", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn options2()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title ?pet
	WHERE {
		?s v:fn ?name .
		OPTIONAL {?s v:honorific-prefix ?title} .
		OPTIONAL {?s v:pet ?pet}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Eddard Stark", ~"")), (~"title", StringValue(~"Lord", ~""))],
		~[(~"name", StringValue(~"Jon Snow", ~"")), (~"pet", StringValue(~"Ghost", ~""))],
		~[(~"name", StringValue(~"Sandor Clegane", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

// Note that operators and functions have their own unit tests so there isn't a lot
// we have to do here.
#[test]
fn filter_constant()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (?age = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_typed_literal()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (?age = \"19\"^^xsd:integer)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_non_ebv()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (?agge = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	
	assert check_solution_err(store, expr, ~"=: ?agge was not bound.");
}

#[test]
fn filter_binary()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (?age = 18 + 5 - 4)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_bound()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (BOUND (?age) && ?age = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_if()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER IF(?s = got:Eddard_Stark, ?age = 45, ?age = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark")))],
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_coalesce()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (COALESCE(?x, ?age) = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_term_fn()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (STR(?age) = \"19\")
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_str_fn()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER CONTAINS(STR(?s), \"_S\")
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark")))],
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_numeric()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s
	WHERE {
		?s v:age ?age .
		FILTER (ABS(?age) = 19)
	}";
	let store = test_data::got_cast3();
	store.add(~"got:Eddard_Stark", ~[
		(~"v:age", IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", IntValue(-19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Jon_Snow")))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn filter_optional()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title ?nick
	WHERE {
		?s v:fn ?name .
		OPTIONAL {?s v:nickname ?nick . FILTER CONTAINS(?nick, \" \")}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Eddard Stark", ~""))],
		~[(~"name", StringValue(~"Jon Snow", ~"")), (~"nick", StringValue(~"Lord Snow", ~""))],
		~[(~"name", StringValue(~"Sandor Clegane", ~"")), (~"nick", StringValue(~"The Hound", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn order_by()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s ?o
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s) && !ISBLANK(?o))
	} ORDER BY ?s ?o";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Lord", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Ned", ~""))],
		
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Ghost", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Jon Snow", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Lord Snow", ~""))],
		
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"Sandor Clegane", ~""))],
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"The Hound", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn bad_order_by()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s ?o
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s) && !ISBLANK(?o))
	} ORDER BY (?s + ?o)";
	let store = test_data::got_cast3();
	
	assert check_solution_err(store, expr, ~"<: +: expected numeric value but found IriValue(~\"http://awoiaf.westeros.org/index.php/Sandor_Clegane\").");
}

#[test]
fn order_by_desc()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s ?o
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s) && !ISBLANK(?o))
	} ORDER BY ASC(?s) DESC(?o)";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Ned", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Lord", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Lord Snow", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Jon Snow", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Ghost", ~""))],
		
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"The Hound", ~""))],
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"Sandor Clegane", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn limit()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s ?o
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s) && !ISBLANK(?o))
	} ORDER BY ?s ?o LIMIT 4";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Lord", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Ned", ~""))],
		
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Ghost", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn big_limit()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?s ?o
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s) && !ISBLANK(?o))
	} ORDER BY ?s ?o LIMIT 400";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Eddard Stark", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Lord", ~""))],
		~[(~"s", IriValue(got(~"Eddard_Stark"))), (~"o", StringValue(~"Ned", ~""))],
		
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Ghost", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Jon Snow", ~""))],
		~[(~"s", IriValue(got(~"Jon_Snow"))), (~"o", StringValue(~"Lord Snow", ~""))],
		
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"Sandor Clegane", ~""))],
		~[(~"s", IriValue(got(~"Sandor_Clegane"))), (~"o", StringValue(~"The Hound", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn bind()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?d
	WHERE {
		got:Eddard_Stark v:honorific-prefix ?o .
		BIND (CONCAT(?o, ?o) AS ?d)
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"d", StringValue(~"LordLord", ~""))],
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn extensions()
{
	let expr = ~"SELECT ?sp ?pp
	WHERE {
		?s ?p ?o .
		BIND(rrdf:pname(?s) AS ?sp) .
		BIND(rrdf:pname(?p) AS ?pp) 
	}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"sp", StringValue(~"got:Eddard_Stark", ~"")), (~"pp", StringValue(~"v:fn", ~""))],
		~[(~"sp", StringValue(~"got:Eddard_Stark", ~"")), (~"pp", StringValue(~"v:nickname", ~""))]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn distinct()
{
	let expr = ~"SELECT DISTINCT ?s
	WHERE {
		?s ?p ?o .
		FILTER (!ISBLANK(?s))
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"s", IriValue(got(~"Eddard_Stark")))],
		~[(~"s", IriValue(got(~"Jon_Snow")))],
		~[(~"s", IriValue(got(~"Sandor_Clegane")))],
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn pname_with_blank()
{
	let expr = ~"
		SELECT DISTINCT
			?name
		WHERE
		{
			?subject ?predicate ?object .
			BIND(rrdf:pname(?subject) AS ?name) .
		} ORDER BY ?name";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"_:jon-org-1", ~""))],
		~[(~"name", StringValue(~"_:ned-org-0", ~""))],
		~[(~"name", StringValue(~"got:Eddard_Stark", ~""))],
		~[(~"name", StringValue(~"got:Jon_Snow", ~""))],
		~[(~"name", StringValue(~"got:Sandor_Clegane", ~""))],
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn animals1()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?subject
	WHERE
	{
		?subject wiki:phylum \"chordata\" .
		?subject wiki:family \"ursidae\"
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"subject", IriValue(wiki(~"grizzly")))],
	]};
	assert check_solution(store, expr, expected);
}

#[test]
fn animals2()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?phylum ?family
	WHERE
	{
		?subject wiki:phylum ?phylum .
		?subject wiki:family ?family
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"phylum", StringValue(~"arthropoda", ~"")), (~"family", StringValue(~"theridiidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"salmonidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"orycteropodidae", ~""))],
		~[(~"phylum", StringValue(~"arthropoda", ~"")), (~"family", StringValue(~"lampyridae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"giraffidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"ursidae", ~""))],
	]};
	assert check_solution(store, expr, expected);
}

#[test]
fn animals3()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?phylum ?family
	WHERE
	{
		?subject wiki:family ?family .
		?subject wiki:phylum ?phylum .
		?subject wiki:class \"mammalia\"
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"orycteropodidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"giraffidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"ursidae", ~""))],
	]};
	assert check_solution(store, expr, expected);
}

#[test]
fn animals4()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?phylum ?family ?foo
	WHERE
	{
		?subject wiki:family ?family .
		?subject wiki:phylum ?phylum .
		?subject wiki:class \"mammalia\" .
		OPTIONAL
		{
			?subject wiki:foobar ?foo
		}
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"orycteropodidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"giraffidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"ursidae", ~""))],
	]};
	assert check_solution(store, expr, expected);
}

#[test]
fn animals5()
{
	let expr = ~"
	PREFIX wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?phylum ?family ?habitat
	WHERE
	{
		?subject wiki:family ?family .
		?subject wiki:phylum ?phylum .
		?subject wiki:class \"mammalia\" .
		OPTIONAL
		{
			?subject wiki:habitat ?habitat
		}
	}";
	
	let store = test_data::animals();
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"orycteropodidae", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"giraffidae", ~"")), (~"habitat", StringValue(~"savannah", ~""))],
		~[(~"phylum", StringValue(~"chordata", ~"")), (~"family", StringValue(~"ursidae", ~""))],
	]};
	assert check_solution(store, expr, expected);
}

#[test]
fn blank_query1()
{
	let expr = ~"
	PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT
		?b ?name ?unit
	WHERE
	{
		?b v:organisation-name ?name .
		?b v:organisation-unit ?unit
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"b", BlankValue(~"_:ned-org-0")), (~"name", StringValue(~"Small Council", ~"")), (~"unit", StringValue(~"Hand", ~""))],
		~[(~"b", BlankValue(~"_:jon-org-1")), (~"name", StringValue(~"Night's Watch", ~"")), (~"unit", StringValue(~"Stewards", ~""))],
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn bad_optional()
{
	let expr = ~"
	PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v:    <http://www.w3.org/2006/vcard/ns#>
	SELECT
		?name ?bogus ?pet
	WHERE
	{
		?s v:fn ?name .
		?s v:bogus ?bogus .
		OPTIONAL {?s v:pet ?pet}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], rows: ~[]};
	
	assert check_solution(store, expr, expected);
}
