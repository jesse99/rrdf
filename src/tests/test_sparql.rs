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
fn trivial()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("fn"))), ("o", string_value("Eddard Stark", ""))],
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("nickname"))), ("o", string_value("Ned", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn out_of_order()
{
	let expr = "SELECT ?o ?s ?p WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("o", string_value("Eddard Stark", "")), ("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("fn")))],
		[("o", string_value("Ned", "")), ("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_names()
{
	let expr = "SELECT ?subject ?p ?obj WHERE {?subject ?p ?obj}";
	let store = test_data::got_cast1();
	let expected = [
		[("subject", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("fn"))), ("obj", string_value("Eddard Stark", ""))],
		[("subject", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("nickname"))), ("obj", string_value("Ned", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn keyword_case()
{
	let expr = "SeLecT ?s ?p ?o where {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("fn"))), ("o", string_value("Eddard Stark", ""))],
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("nickname"))), ("o", string_value("Ned", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn duplicate_select_variables()
{
	let expr = "SELECT ?s ?s ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(store, expr, "Select clause has duplicates: s");
}

#[test]
fn duplicate_where_variables()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?s ?o}";
	let store = test_data::got_cast1();
	
	assert check_solution_err(store, expr, "Binding s was set more than once.");
}

#[test]
fn unbound_variable()
{
	let expr = "SELECT ?s ?p ?z WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("fn")))],
		[("s", iri_value(got("Eddard_Stark"))), ("p", iri_value(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn no_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"Peter Pan\"}";
	let store = test_data::got_cast1();
	let expected = [];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn comment()
{
	let expr = "SELECT ?s ?p #your comment here
	WHERE {	# yet another comment
		?s ?p \"Peter Pan\"
	}";
	let store = test_data::got_cast1();
	let expected = [];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn simple_path()
{
	let expr = "SELECT ?org
	WHERE {
		<http://awoiaf.westeros.org/index.php/Eddard_Stark> <http://www.w3.org/2006/vcard/ns#org> ?z .
		?z <http://www.w3.org/2006/vcard/ns#organisation-name> ?org
	}";
	let store = test_data::got_cast3();
	let expected = [
		[("org", string_value("Small Council", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn select_all()
{
	let expr = "SELECT *
	WHERE {
		<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?o
	}";
	let store = test_data::got_cast3();
	let expected = [
		[("p", iri_value(v("fn"))), ("o", string_value("Sandor Clegane", ""))],
		[("p", iri_value(v("nickname"))), ("o", string_value("The Hound", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn prefixes()
{
	let expr = "PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?org
	WHERE {
		got:Eddard_Stark v:org ?z .
		?z v:organisation-name ?org
	}";
	let store = test_data::got_cast3();
	let expected = [
		[("org", string_value("Small Council", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn options1()
{
	let expr = "PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title ?pet
	WHERE {
		?s v:fn ?name .
		OPTIONAL {
			?s v:honorific-prefix ?title
		}
	}";
	let store = test_data::got_cast3();
	let expected = [
		[("name", string_value("Eddard Stark", "")), ("title", string_value("Lord", ""))],
		[("name", string_value("Jon Snow", ""))],
		[("name", string_value("Sandor Clegane", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn options2()
{
	let expr = "PREFIX got: <http://awoiaf.westeros.org/index.php/>
	PREFIX v: <http://www.w3.org/2006/vcard/ns#>
	SELECT ?name ?title ?pet
	WHERE {
		?s v:fn ?name .
		OPTIONAL {?s v:honorific-prefix ?title} .
		OPTIONAL {?s v:pet ?pet}
	}";
	let store = test_data::got_cast3();
	let expected = [
		[("name", string_value("Eddard Stark", "")), ("title", string_value("Lord", ""))],
		[("name", string_value("Jon Snow", "")), ("pet", string_value("Ghost", ""))],
		[("name", string_value("Sandor Clegane", ""))]
	];
	
	assert check_solution(store, expr, expected);
}

