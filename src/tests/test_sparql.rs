import test_helpers::*;

#[test]
fn trivial()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = {names: ["s", "p", "o"], rows: [
		ref_uri_str("got:Eddard_Stark", "v:fn", "Eddard Stark"),
		ref_uri_str("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn out_of_order()
{
	let expr = "SELECT ?o ?s ?p WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = {names: ["o", "s", "p"], rows: [
		str_ref_uri("Eddard Stark", "got:Eddard_Stark", "v:fn"),
		str_ref_uri("Ned", "got:Eddard_Stark", "v:nickname")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_names()
{
	let expr = "SELECT ?subject ?p ?obj WHERE {?subject ?p ?obj}";
	let store = test_data::got_cast1();
	let expected = {names: ["subject", "p", "obj"], rows: [
		ref_uri_str("got:Eddard_Stark", "v:fn", "Eddard Stark"),
		ref_uri_str("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn keyword_case()
{
	let expr = "SeLecT ?s ?p ?o where {?s ?p ?o}";
	let store = test_data::got_cast1();
	let expected = {names: ["s", "p", "o"], rows: [
		ref_uri_str("got:Eddard_Stark", "v:fn", "Eddard Stark"),
		ref_uri_str("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
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
	let expected = {names: ["s", "p", "z"], rows: [
		ref_uri_none("got:Eddard_Stark", "v:fn"),
		ref_uri_none("got:Eddard_Stark", "v:nickname")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn string1_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p 'Ned'}";
	let store = test_data::got_cast1();
	let expected = {names: ["s", "p"], rows: [
		ref_uri("got:Eddard_Stark", "v:nickname")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn string2_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"Ned\"}";
	let store = test_data::got_cast1();
	let expected = {names: ["s", "p"], rows: [
		ref_uri("got:Eddard_Stark", "v:nickname")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string1_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p '''Name\nTitle'''}";
	let store = test_data::got_cast1();
	add_triples(store, [
		{subject: "got:Some_Guy", predicate: "v:fn", object: {value: "Name\nTitle", kind: "xsd:string", lang: ""}}
		]);
	
	let expected = {names: ["s", "p"], rows: [
		ref_uri("got:Some_Guy", "v:fn")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn long_string2_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"\"\"Bob \"Bob\n\"\"\"}";
	let store = test_data::got_cast1();
	add_triples(store, [
		{subject: "got:Some_Guy", predicate: "v:fn", object: {value: "Bob \"Bob\n", kind: "xsd:string", lang: ""}}
		]);
	
	let expected = {names: ["s", "p"], rows: [
		ref_uri("got:Some_Guy", "v:fn")
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn no_match()
{
	let expr = "SELECT ?s ?p WHERE {?s ?p \"Peter Pan\"}";
	let store = test_data::got_cast1();
	let expected = {names: ["s", "p"], rows: []};
	
	assert check_solution(store, expr, expected);
}

fn fancy_types() -> store
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "x", path: "http://blah#"}
		]);
	
	add_triples(store, [
		{subject: "x:Hans", predicate: "x:greeting", object: {value: "guten tag", kind: "xsd:string", lang: "de"}},
		{subject: "x:Jones", predicate: "x:greeting", object: {value: "guten tag", kind: "xsd:string", lang: "en-US"}}
		]);
	ret store;
}

#[test]
fn language_tags()
{
	let expr = "SELECT ?s WHERE {?s ?p \"guten tag\"@en-US}";
	let store = fancy_types();
	let expected = {names: ["s"], rows: [
		[option::some({value: "x:Jones", kind: "xsd:anyURI", lang: ""})]
	]};
	
	assert check_solution(store, expr, expected);
}

// TODO:
// iri literals
// matching specific subject
// int literal
// float literal
// boolean literal
// NIL literal
// maybe datetime literals
// would it make sense to use NIL instead of option::none?
// might want a test_literals.rs file
