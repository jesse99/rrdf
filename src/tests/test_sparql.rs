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
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("fn"))), ("o", create_str("Eddard Stark"))],
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname"))), ("o", create_str("Ned"))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn out_of_order()
{
	let expr = "SELECT ?o ?s ?p WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("o", create_str("Eddard Stark")), ("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("fn")))],
		[("o", create_str("Ned")), ("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname")))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_names()
{
	let expr = "SELECT ?subject ?p ?obj WHERE {?subject ?p ?obj}";
	let store = test_data::got_cast1();
	let expected = [
		[("subject", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("fn"))), ("obj", create_str("Eddard Stark"))],
		[("subject", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname"))), ("obj", create_str("Ned"))]
	];
	
	assert check_solution(store, expr, expected);
}

#[test]
fn keyword_case()
{
	let expr = "SeLecT ?s ?p ?o where {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = [
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("fn"))), ("o", create_str("Eddard Stark"))],
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname"))), ("o", create_str("Ned"))]
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
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("fn")))],
		[("s", create_uri(got("Eddard_Stark"))), ("p", create_uri(v("nickname")))]
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
		[("org", create_str("Small Council"))]
	];
	
	assert check_solution(store, expr, expected);
}

