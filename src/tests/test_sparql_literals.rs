import std::map::*;
import test_helpers::*;

fn got(s: ~str) -> ~str
{
	~"http://awoiaf.westeros.org/index.php/" + s
}

fn v(s: ~str) -> ~str
{
	~"http://www.w3.org/2006/vcard/ns#" + s
}

#[test]
fn string1_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p 'Ned'}";
	let store = test_data::got_cast1();
	let expected = ~[
		~[(~"s", iri_value(got(~"Eddard_Stark"))), (~"p", iri_value(v(~"nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn string2_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p \"Ned\"}";
	let store = test_data::got_cast1();
	let expected = ~[
		~[(~"s", iri_value(got(~"Eddard_Stark"))), (~"p", iri_value(v(~"nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string1_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p '''Name\nTitle'''}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:fn", string_value(~"Name\nTitle", ~""))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Some_Guy"))), (~"p", iri_value(v(~"fn")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string2_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p \"\"\"Bob \"Bob\n\"\"\"}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:fn", string_value(~"Bob \"Bob\n", ~""))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Some_Guy"))), (~"p", iri_value(v(~"fn")))]
	];
	
	assert check_solution(store, expr, expected);
}

fn fancy_types() -> store
{
	let store = create_store(~[
		{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		{prefix: ~"x", path: ~"http://blah#"}
		], @std::map::str_hash());
	
	store.add(~"x:Hans", ~[(~"x:greeting", string_value(~"guten tag", ~"de"))]);
	store.add(~"x:Jones", ~[(~"x:greeting", string_value(~"guten tag", ~"en-US"))]);
	ret store;
}

#[test]
fn language_tags()
{
	let expr = ~"SELECT ?s WHERE {?s ?p \"guten tag\"@en-US}";
	let store = fancy_types();
	let expected = ~[
		~[(~"s", iri_value(~"http://blah#Jones"))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn iri_match()
{
	let expr = ~"SELECT ?s WHERE {?s <http://www.w3.org/2006/vcard/ns#nickname> ?z}";
	let store = test_data::got_cast3();
	let expected = ~[
		~[(~"s", iri_value(got(~"Eddard_Stark")))],
		~[(~"s", iri_value(got(~"Jon_Snow")))],
		~[(~"s", iri_value(got(~"Sandor_Clegane")))]
	];
	
	assert check_solution(store, expr, expected);
}

// This represents a special case in iterate_matches.
#[test]
fn subject_match()
{
	let expr = ~"SELECT ?p WHERE {<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?z}";
	let store = test_data::got_cast3();
	let expected = ~[
		~[(~"p", iri_value(v(~"fn")))],
		~[(~"p", iri_value(v(~"nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn strings_dont_match_uris()
{
	let expr = ~"SELECT ?p WHERE {\"got:Sandor_Clegane\" ?p ?z}";
	let store = test_data::got_cast3();
	let expected = ~[];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn typed_literal_match()
{
	let expr = ~"SELECT ?s ?p WHERE {?s ?p \"Ned\"^^<http://www.w3.org/2001/XMLSchema#string>}";
	let store = test_data::got_cast1();
	let expected = ~[
		~[(~"s", iri_value(got(~"Eddard_Stark"))), (~"p", iri_value(v(~"nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn typed_literal_match2()
{
	let expr = ~"
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
SELECT 
	?s ?p 
WHERE 
{
	?s ?p \"Ned\"^^xsd:string
}";
	let store = test_data::got_cast1();
	let expected = ~[
		~[(~"s", iri_value(got(~"Eddard_Stark"))), (~"p", iri_value(v(~"nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn int_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p 23}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:age", int_value(23i64))
	]);
	store.add(~"got:Another_Guy", ~[
		(~"v:age", int_value(23i64))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Another_Guy")))],
		~[(~"s", iri_value(got(~"Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn signed_int_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p -23}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[(~"v:age", int_value(23i64))]);
	store.add(~"got:Another_Guy", ~[(~"v:age", int_value(-23i64))]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Another_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn decimal_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p 3.14}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[(~"v:age", float_value(3.14f64))]);
	store.add(~"got:Another_Guy", ~[(~"v:age", float_value(3.14f64))]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Another_Guy")))],
		~[(~"s", iri_value(got(~"Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn signed_decimal_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p -3.14}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:age", float_value(3.14f64))
	]);
	store.add(~"got:Another_Guy", ~[
		(~"v:age", float_value(-3.14f64))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Another_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn double_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p 314e-2}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:age", float_value(3.14f64))
	]);
	store.add(~"got:Another_Guy", ~[
		(~"v:age", float_value(3.14f64))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"Another_Guy")))],
		~[(~"s", iri_value(got(~"Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn boolean_literal()
{
	let expr = ~"SELECT ?s WHERE {?s ?p false}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:male", bool_value(true))
	]);
	store.add(~"got:A_Woman", ~[
		(~"v:male", bool_value(false))
	]);
	
	let expected = ~[
		~[(~"s", iri_value(got(~"A_Woman")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn datetime()
{
	// TODO: enable the second case once bug #2637 is fixed
	let expr = ~"SELECT ?s WHERE {?s ?p \"1999-05-31T13:10:00-05:00\"^^<http://www.w3.org/2001/XMLSchema#dateTime>}";
	let store = test_data::got_cast1();
	store.add(~"got:Some_Guy", ~[
		(~"v:born", literal_to_object(~"1999-05-31T13:10:00-05:00", ~"http://www.w3.org/2001/XMLSchema#dateTime", ~""))
	]);
//	store.add(~"got:A_Woman", ~[
//		(~"v:born", literal_to_object(~"1999-05-31T14:10:00-04:00", ~"http://www.w3.org/2001/XMLSchema#dateTime", ~""))
//	]);
	store.add(~"got:A_Dude", ~[
		(~"v:born", literal_to_object(~"1999-05-31T13:22:00-05:00", ~"http://www.w3.org/2001/XMLSchema#dateTime", ~""))
	]);
	
	let expected = ~[
//		~[(~"s", iri_value(got(~"A_Woman")))],
		~[(~"s", iri_value(got(~"Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}
