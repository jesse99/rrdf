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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: plain_literal("Eddard Stark", "")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: plain_literal("Ned", "")}
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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: plain_literal("Eddard Stark", "")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: plain_literal("Ned", "")},
		{subject: "got:Eddard_Stark", predicate: "v:honorific-prefix", object: plain_literal("Lord", "")},
		{subject: "got:Eddard_Stark", predicate: "v:org", object: reference("_:ned-org")},
		{subject: "_:ned-org", predicate: "v:organisation-name", object: plain_literal("Small Council", "")},
		{subject: "_:ned-org", predicate: "v:organisation-unit", object: plain_literal("Hand", "")},
		
		{subject: "got:Jon_Snow", predicate: "v:fn", object: plain_literal("Jon Snow", "")},
		{subject: "got:Jon_Snow", predicate: "v:nickname", object: plain_literal("Lord Snow", "")},
		{subject: "got:Jon_Snow", predicate: "v:org", object: reference("_:jon-org")},
		{subject: "_:jon-org", predicate: "v:organisation-name", object: plain_literal("Night's Watch", "")},
		{subject: "_:jon-org", predicate: "v:organisation-unit", object: plain_literal("Stewards", "")},
		
		{subject: "got:Sandor_Clegane", predicate: "v:fn", object: plain_literal("Sandor Clegane", "")},
		{subject: "got:Sandor_Clegane", predicate: "v:nickname", object: plain_literal("The Hound", "")}
		]);
	ret store;
}
