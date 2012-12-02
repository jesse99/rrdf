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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p", ~"o"], num_selected: 3, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"fn")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"nickname")), @StringValue(~"Ned", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn out_of_order()
{
	let expr = ~"SELECT ?o ?s ?p WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"o", ~"s", ~"p"], num_selected: 3, rows: ~[
		~[@StringValue(~"Eddard Stark", ~""), @IriValue(got(~"Eddard_Stark")), @IriValue(v(~"fn"))],
		~[@StringValue(~"Ned", ~""), @IriValue(got(~"Eddard_Stark")), @IriValue(v(~"nickname"))]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn long_names()
{
	let expr = ~"SELECT ?subject ?p ?obj WHERE {?subject ?p ?obj}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"subject", ~"p", ~"obj"], num_selected: 3, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"fn")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"nickname")), @StringValue(~"Ned", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn keyword_case()
{
	let expr = ~"SeLecT ?s ?p ?o where {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p", ~"o"], num_selected: 3, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"fn")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"nickname")), @StringValue(~"Ned", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn duplicate_select_variables()
{
	let expr = ~"SELECT ?s ?s ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(&store, expr, ~"Select clause has duplicates: s");
}

#[test]
fn duplicate_where_variables()
{
	let expr = ~"SELECT ?s ?p ?o WHERE {?s ?s ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(&store, expr, ~"Binding ?s was set more than once.");
}

#[test]
fn unbound_variable()
{
	let expr = ~"SELECT ?s ?p ?z WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p", ~"z", ~"o"], num_selected: 3, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"fn")), @UnboundValue],
		~[@IriValue(got(~"Eddard_Stark")), @IriValue(v(~"nickname")), @UnboundValue]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn no_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p \"Peter Pan\"}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p"], num_selected: 2, rows: ~[]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn comment()
{
	let expr = ~"SELECT ?s ?p #your comment here
	WHERE {	# yet another comment
		?s ?p \"Peter Pan\"
	}";
	let store = test_data::got_cast1();
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p"], num_selected: 2, rows: ~[]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"org", ~"z"], num_selected: 1, rows: ~[
		~[@StringValue(~"Small Council", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	
	let store = Store(~[Namespace {prefix: ~"wiki", path: ~"http://en.wikipedia.org/wiki/"}], &HashMap());
	store.add(~"wiki:giraffe", ~[
		(~"wiki:phylum", @StringValue(~"chordata", ~"")),
		(~"wiki:class", @StringValue(~"mammalia", ~"")),
	]);
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"subject"], num_selected: 1, rows: ~[]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"subject"], num_selected: 1, rows: ~[]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"subject", ~"family"], num_selected: 2, rows: ~[]};
	assert check_eval(&store, expr, &expected);
}

#[test]
fn select_all()
{
	let expr = ~"SELECT *
	WHERE {
		<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?o
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], bindings: ~[~"p", ~"o"], num_selected: 2, rows: ~[
		~[@IriValue(v(~"fn")), @StringValue(~"Sandor Clegane", ~"")],
		~[@IriValue(v(~"nickname")), @StringValue(~"The Hound", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"org", ~"z"], num_selected: 1, rows: ~[
		~[@StringValue(~"Small Council", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn options1()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title
	WHERE {
		?s v:fn ?name .
		OPTIONAL {
			?s v:honorific-prefix ?title
		}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"title", ~"s"], num_selected: 2, rows: ~[
		~[@StringValue(~"Eddard Stark", ~""), @StringValue(~"Lord", ~"")],
		~[@StringValue(~"Jon Snow", ~""), @UnboundValue],
		~[@StringValue(~"Sandor Clegane", ~""), @UnboundValue]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"title", ~"pet", ~"s"], num_selected: 3, rows: ~[
		~[@StringValue(~"Eddard Stark", ~""), @StringValue(~"Lord", ~""), @UnboundValue],
		~[@StringValue(~"Jon Snow", ~""), @UnboundValue, @StringValue(~"Ghost", ~"")],
		~[@StringValue(~"Sandor Clegane", ~""), @UnboundValue, @UnboundValue]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	
	assert check_solution_err(&store, expr, ~"=: ?agge was not bound.");
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Eddard_Stark"))],
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Eddard_Stark"))],
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
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
		(~"v:age", @IntValue(45i64))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:age", @IntValue(-19i64))
	]);
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:age", @IntValue(35i64))
	]);
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"age"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Jon_Snow"))]
	]};
	
	assert check_eval(&store, expr, &expected);
}

#[test]
fn filter_optional()
{
	let expr = ~"PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?nick
	WHERE {
		?s v:fn ?name .
		OPTIONAL {?s v:nickname ?nick . FILTER CONTAINS(?nick, \" \")}
	}";
	let store = test_data::got_cast3();
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"nick", ~"s"], num_selected: 2, rows: ~[
		~[@StringValue(~"Eddard Stark", ~""), @UnboundValue],
		~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
		~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"o", ~"p"], num_selected: 2, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Lord", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Ned", ~"")],
		
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Ghost", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Jon Snow", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Lord Snow", ~"")],
		
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"Sandor Clegane", ~"")],
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"The Hound", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	
	assert check_solution_err(&store, expr, ~"<: +: expected numeric value but found IriValue(~\"http://awoiaf.westeros.org/index.php/Eddard_Stark\").");
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"o", ~"p"], num_selected: 2, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Ned", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Lord", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Eddard Stark", ~"")],
		
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Lord Snow", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Jon Snow", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Ghost", ~"")],
		
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"The Hound", ~"")],
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"Sandor Clegane", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"o", ~"p"], num_selected: 2, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Lord", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Ned", ~"")],
		
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Ghost", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"o", ~"p"], num_selected: 2, rows: ~[
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Eddard Stark", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Lord", ~"")],
		~[@IriValue(got(~"Eddard_Stark")), @StringValue(~"Ned", ~"")],
		
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Ghost", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Jon Snow", ~"")],
		~[@IriValue(got(~"Jon_Snow")), @StringValue(~"Lord Snow", ~"")],
		
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"Sandor Clegane", ~"")],
		~[@IriValue(got(~"Sandor_Clegane")), @StringValue(~"The Hound", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"d", ~"o"], num_selected: 1, rows: ~[
		~[@StringValue(~"LordLord", ~"")],
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"sp", ~"pp", ~"s", ~"p", ~"o"], num_selected: 2, rows: ~[
		~[@StringValue(~"got:Eddard_Stark", ~""), @StringValue(~"v:fn", ~"")],
		~[@StringValue(~"got:Eddard_Stark", ~""), @StringValue(~"v:nickname", ~"")]
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"s", ~"p", ~"o"], num_selected: 1, rows: ~[
		~[@IriValue(got(~"Eddard_Stark"))],
		~[@IriValue(got(~"Jon_Snow"))],
		~[@IriValue(got(~"Sandor_Clegane"))],
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"subject", ~"predicate", ~"object"], num_selected: 1, rows: ~[
		~[@StringValue(~"_:jon-org-1", ~"")],
		~[@StringValue(~"_:ned-org-0", ~"")],
		~[@StringValue(~"got:Eddard_Stark", ~"")],
		~[@StringValue(~"got:Jon_Snow", ~"")],
		~[@StringValue(~"got:Sandor_Clegane", ~"")],
	]};
	
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"subject"], num_selected: 1, rows: ~[
		~[@IriValue(wiki(~"grizzly"))],
	]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"phylum", ~"family", ~"subject"], num_selected: 2, rows: ~[
		~[@StringValue(~"arthropoda", ~""), @StringValue(~"theridiidae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"salmonidae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"orycteropodidae", ~"")],
		~[@StringValue(~"arthropoda", ~""), @StringValue(~"lampyridae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"giraffidae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"ursidae", ~"")],
	]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"phylum", ~"family", ~"subject"], num_selected: 2, rows: ~[
		~[@StringValue(~"chordata", ~""), @StringValue(~"orycteropodidae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"giraffidae", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"ursidae", ~"")],
	]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"phylum", ~"family", ~"foo", ~"subject"], num_selected: 3, rows: ~[
		~[@StringValue(~"chordata", ~""), @StringValue(~"orycteropodidae", ~""), @UnboundValue],
		~[@StringValue(~"chordata", ~""), @StringValue(~"giraffidae", ~""), @UnboundValue],
		~[@StringValue(~"chordata", ~""), @StringValue(~"ursidae", ~""), @UnboundValue],
	]};
	assert check_eval(&store, expr, &expected);
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
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"phylum", ~"family", ~"habitat", ~"subject"], num_selected: 3, rows: ~[
		~[@StringValue(~"chordata", ~""), @StringValue(~"orycteropodidae", ~""), @UnboundValue],
		~[@StringValue(~"chordata", ~""), @StringValue(~"giraffidae", ~""), @StringValue(~"savannah", ~"")],
		~[@StringValue(~"chordata", ~""), @StringValue(~"ursidae", ~""), @UnboundValue],
	]};
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"b", ~"name", ~"unit"], num_selected: 3, rows: ~[
		~[@BlankValue(~"_:ned-org-0"), @StringValue(~"Small Council", ~""), @StringValue(~"Hand", ~"")],
		~[@BlankValue(~"_:jon-org-1"), @StringValue(~"Night's Watch", ~""), @StringValue(~"Stewards", ~"")],
	]};
	
	assert check_eval(&store, expr, &expected);
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
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"bogus", ~"pet", ~"s"], num_selected: 3, rows: ~[]};
	
	assert check_eval(&store, expr, &expected);
}
