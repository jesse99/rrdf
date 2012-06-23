import std::map::*;
import test_helpers::*;

fn got(s: str) -> str
{
	"http://awoiaf.westeros.org/index.php/" + s
}

fn v(s: str) -> str
{
	"http://www.w3.org/2006/vcard/ns#" + s
}

#[test]
fn string1_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p 'Ned'}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn string2_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"Ned\"}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string1_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p '''Name\nTitle'''}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:fn", create_str("Name\nTitle"))
	]);
	
	let expected = [
		[("s", create_uri(got("Some_Guy"))), ("p", create_uri(v("fn")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string2_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"\"\"Bob \"Bob\n\"\"\"}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:fn", create_str("Bob \"Bob\n"))
	]);
	
	let expected = [
		[("s", create_uri(got("Some_Guy"))), ("p", create_uri(v("fn")))]
	];
	
	assert check_solution(store, expr, expected);
}

fn fancy_types() -> store
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "x", path: "http://blah#"}
		]);
	
	store.add("x:Hans", [("x:greeting", create_lang("guten tag", "de"))]);
	store.add("x:Jones", [("x:greeting", create_lang("guten tag", "en-US"))]);
	ret store;
}

#[test]
fn language_tags()
{
	let expr = "SELECT ?s WHERE {?s ?p \"guten tag\"@en-US}";
	let store = fancy_types();
	let expected = [
		[("s", create_uri("http://blah#Jones"))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn iri_match()
{
	let expr = "SELECT ?s WHERE {?s <http://www.w3.org/2006/vcard/ns#nickname> ?z}";
	let store = test_data::got_cast3();
	let expected = [
		[("s", create_uri(got("Eddard_Stark")))],
		[("s", create_uri(got("Jon_Snow")))],
		[("s", create_uri(got("Sandor_Clegane")))]
	];
	
	assert check_solution(store, expr, expected);
}

// This represents a special case in iterate_matches.
#[test]
fn subject_match()
{
	let expr = "SELECT ?p WHERE {<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?z}";
	let store = test_data::got_cast3();
	let expected = [
		[("p", create_uri(v("fn")))],
		[("p", create_uri(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn strings_dont_match_uris()
{
	let expr = "SELECT ?p WHERE {\"got:Sandor_Clegane\" ?p ?z}";
	let store = test_data::got_cast3();
	let expected = [];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn typed_literal_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"Ned\"^^<http://www.w3.org/2001/XMLSchema#string>}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn int_literal()
{
	let expr = "SELECT ?s WHERE {?s ?p 23}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:age", create_int(23))
	]);
	store.add("got:Another_Guy", [
		("v:age", create_int(23))
	]);
	
	let expected = [
		[("s", create_uri(got("Another_Guy")))],
		[("s", create_uri(got("Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

// TODO: Not sure what is supposed to happen here. According to 18.6 and http://www.w3.org/TR/sparql11-entailment/
// we're apparently supposed to use the 'simple entailment relation between RDF graphs' in http://www.w3.org/TR/rdf-mt/#entail
// which looks far from simple. 
#[test]
fn signed_int_literal()
{
	let expr = "SELECT ?s WHERE {?s ?p -23}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [("v:age", create_int(23))]);
	store.add("got:Another_Guy", [("v:age", create_typed("-23", "xsd:long"))]);
	
	let expected = [];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn decimal_literal()
{
	let expr = "SELECT ?s WHERE {?s ?p 3.14}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [("v:age", create_typed("3.14", "xsd:float"))]);
	store.add("got:Another_Guy", [("v:age", create_typed("3.14", "xsd:double"))]);
	
	let expected = [
		[("s", create_uri(got("Another_Guy")))]
		//[("s", create_uri(got("Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn signed_decimal_literal()
{
	let expr = "SELECT ?s WHERE {?s ?p -3.14}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:age", create_typed("3.14", "xsd:double"))
	]);
	store.add("got:Another_Guy", [
		("v:age", create_typed("-3.14", "xsd:double"))
	]);
	
	let expected = [
		[("s", create_uri(got("Another_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}

// TODO: is this supposed to work?
//#[test]
//fn double_literal()
//{
//	let expr = "SELECT ?s WHERE {?s ?p 314e-2}";
//	let store = test_data::got_cast1();
//	store.add("got:Some_Guy", [
//		make_typed("v:age", "3.14", "xsd:double")
//	]);
//	store.add("got:Another_Guy", [
//		make_typed("v:age", "3.14", "xsd:double")
//	]);
//	
//	let expected = [
//		[("s", create_uri(got("Another_Guy")))],
//		[("s", create_uri(got("Some_Guy")))]
//	];
//	
//	assert check_solution(store, expr, expected);
//}

#[test]
fn boolean_literal()
{
	let expr = "SELECT ?s WHERE {?s ?p false}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:male", create_bool(true))
	]);
	store.add("got:A_Woman", [
		("v:male", create_bool(false))
	]);
	
	let expected = [
		[("s", create_uri(got("A_Woman")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn datetime()
{
	// TODO: enable the second case once bug #2637 is fixed
	let expr = "SELECT ?s WHERE {?s ?p \"1999-05-31T13:10:00-05:00\"^^<http://www.w3.org/2001/XMLSchema#dateTime>}";
	let store = test_data::got_cast1();
	store.add("got:Some_Guy", [
		("v:born", create_typed("1999-05-31T13:10:00-05:00", "xsd:dateTime"))
	]);
//	store.add("got:A_Woman", [
//		("v:born", create_typed("1999-05-31T14:10:00-04:00", "xsd:dateTime"))
//	]);
	store.add("got:A_Dude", [
		("v:born", create_typed("1999-05-31T13:22:00-05:00", "xsd:dateTime"))
	]);
	
	let expected = [
//		[("s", create_uri(got("A_Woman")))],
		[("s", create_uri(got("Some_Guy")))]
	];
	
	assert check_solution(store, expr, expected);
}
