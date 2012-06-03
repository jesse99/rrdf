fn str_spo(subject: str, property: str, object: str) -> [option<object>]
{
	[option::some(reference(iri(subject))), option::some(anyURI(property)), option::some(string(object))]
}

#[test]
fn trivial()
{
	let expr = "SELECT ?s ?p ?o WHERE {?s ?p ?o}";
	let triples = test_data::got_cast1();
	let expected = {names: ["s", "p", "o"], rows: [
		str_spo("got:Eddard_Stark", "v:fn", "Eddard_Stark"), 
		str_spo("got:Eddard_Stark", "v:nickname", "Ned")
	]};
	
	assert test_helpers::check_result(triples, expr, expected);
}

// TODO:
// make sure failures are printed nicely
// test different cases for keywords
// test bad duplications of binding names