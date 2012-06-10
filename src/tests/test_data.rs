// Hard-coded triple stores used for unit testing. These use a few different schemas:
// 1) http://www.w3.org/TR/vcard-rdf/
// Profile for electronix business cards.
// 2) https://github.com/edumbill/doap/wiki
// Profile used to describe open source software projects.

fn got_cast1() -> store
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "v", path: "http://www.w3.org/2006/vcard/ns#"}
		]);
	
	add_triples(store, [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: typed_literal("Eddard Stark", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: typed_literal("Ned", "xsd:string")}
		]);
	ret store;
}

fn got_cast3() -> store
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "v", path: "http://www.w3.org/2006/vcard/ns#"}
		]);
	
	add_triples(store, [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: typed_literal("Eddard Stark", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: typed_literal("Ned", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:honorific-prefix", object: typed_literal("Lord", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:org", object: reference("_:ned-org")},
		{subject: "_:ned-org", predicate: "v:organisation-name", object: typed_literal("Small Council", "xsd:string")},
		{subject: "_:ned-org", predicate: "v:organisation-unit", object: typed_literal("Hand", "xsd:string")},
		
		{subject: "got:Jon_Snow", predicate: "v:fn", object: typed_literal("Jon Snow", "xsd:string")},
		{subject: "got:Jon_Snow", predicate: "v:nickname", object: typed_literal("Lord Snow", "xsd:string")},
		{subject: "got:Jon_Snow", predicate: "v:org", object: reference("_:jon-org")},
		{subject: "_:jon-org", predicate: "v:organisation-name", object: typed_literal("Night's Watch", "xsd:string")},
		{subject: "_:jon-org", predicate: "v:organisation-unit", object: typed_literal("Stewards", "xsd:string")},
		
		{subject: "got:Sandor_Clegane", predicate: "v:fn", object: typed_literal("Sandor Clegane", "xsd:string")},
		{subject: "got:Sandor_Clegane", predicate: "v:nickname", object: typed_literal("The Hound", "xsd:string")}
		]);
	ret store;
}
