import test_helpers::*;

#[test]
fn trivial()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?p ?o}";
	let triples = test_data::got_cast1();
	let expected = {names: ["s", "p", "o"], rows: [
		ref_uri_str("got:Eddard_Stark", "v:fn", "Eddard Stark"),
		ref_uri_str("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
	assert check_ok(triples, expr, expected);
}

#[test]
fn keyword_case()
{
	let expr = "SeLecT ?s ?p ?o where {?s ?p ?o}";
	let triples = test_data::got_cast1();
	let expected = {names: ["s", "p", "o"], rows: [
		ref_uri_str("got:Eddard_Stark", "v:fn", "Eddard Stark"),
		ref_uri_str("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
	assert check_ok(triples, expr, expected);
}

#[test]
fn duplicate_select_variables()
{
	let expr = "SELECT ?s ?s ?o WHERE {?s ?p ?o}";
	let triples = test_data::got_cast1();
	
	assert check_err(triples, expr, "Select clause has duplicates: s");
}

#[test]
fn duplicate_where_variables()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?s ?o}";
	let triples = test_data::got_cast1();
	
	assert check_err(triples, expr, "Binding s was set more than once.");
}

#[test]
fn unbound_variable()
{
	let expr = "SELECT ?s ?p ?z WHERE {?s ?p ?o}";
	let triples = test_data::got_cast1();
	let expected = {names: ["s", "p", "z"], rows: [
		ref_uri_none("got:Eddard_Stark", "v:fn"),
		ref_uri_none("got:Eddard_Stark", "v:nickname")
	]};
	
	assert check_ok(triples, expr, expected);
}

// TODO:
// test string literal pattern
// test int literal pattern
// test no matches
