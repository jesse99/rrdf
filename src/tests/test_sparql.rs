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

#[test]
fn iri_match()
{
	let expr = "SELECT ?s WHERE {?s <http://www.w3.org/2006/vcard/ns#nickname> ?z}";
	let store = test_data::got_cast3();
	let expected = {names: ["s"], rows: [
		[option::some({value: "got:Eddard_Stark", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "got:Jon_Snow", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "got:Sandor_Clegane", kind: "xsd:anyURI", lang: ""})]
	]};
	
	assert check_solution(store, expr, expected);
}

// This represents a special case in iterate_matches.
#[test]
fn subject_match()
{
	let expr = "SELECT ?p WHERE {<http://awoiaf.westeros.org/index.php/Sandor_Clegane> ?p ?z}";
	let store = test_data::got_cast3();
	let expected = {names: ["p"], rows: [
		[option::some({value: "v:fn", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "v:nickname", kind: "xsd:anyURI", lang: ""})]
	]};
	
	assert check_solution(store, expr, expected);
}


#[test]
fn prefix_name_match()
{
	let expr = "SELECT ?s WHERE {?s v:nickname ?z}";
	let store = test_data::got_cast3();
	let expected = {names: ["s"], rows: [
		[option::some({value: "got:Eddard_Stark", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "got:Jon_Snow", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "got:Sandor_Clegane", kind: "xsd:anyURI", lang: ""})]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn prefix_subject_match()
{
	let expr = "SELECT ?p WHERE {got:Sandor_Clegane ?p ?z}";
	let store = test_data::got_cast3();
	let expected = {names: ["p"], rows: [
		[option::some({value: "v:fn", kind: "xsd:anyURI", lang: ""})],
		[option::some({value: "v:nickname", kind: "xsd:anyURI", lang: ""})]
	]};
	
	assert check_solution(store, expr, expected);
}

#[test]
fn strings_dont_match_uris()
{
	let expr = "SELECT ?p WHERE {\"got:Sandor_Clegane\" ?p ?z}";
	let store = test_data::got_cast3();
	let expected = {names: ["p"], rows: []};
	
	assert check_solution(store, expr, expected);
}

// TODO:
// typed string
// int literal
// float literal
// mixed numbers (including typed literals)
// boolean literal
// NIL literal
// maybe datetime literals
// would it make sense to use NIL instead of option::none?
// might want a test_literals.rs file
// check error reporting for some common cases (and add tags as appropriate)
