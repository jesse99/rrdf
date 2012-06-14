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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: {value: "Eddard Stark", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: {value: "Ned", kind: "xsd:string", lang: ""}}
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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: {value: "Eddard Stark", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: {value: "Ned", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:honorific-prefix", object: {value: "Lord", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:org", object: {value: "_:ned-org", kind: "xsd:anyURI", lang: ""}},
		{subject: "_:ned-org", predicate: "v:organisation-name", object: {value: "Small Council", kind: "xsd:string", lang: ""}},
		{subject: "_:ned-org", predicate: "v:organisation-unit", object: {value: "Hand", kind: "xsd:string", lang: ""}},
		
		{subject: "got:Jon_Snow", predicate: "v:fn", object: {value: "Jon Snow", kind: "xsd:string", lang: ""}},
		{subject: "got:Jon_Snow", predicate: "v:nickname", object: {value: "Lord Snow", kind: "xsd:string", lang: ""}},
		{subject: "got:Jon_Snow", predicate: "v:org", object: {value: "_:jon-org", kind: "xsd:anyURI", lang: ""}},
		{subject: "_:jon-org", predicate: "v:organisation-name", object: {value: "Night's Watch", kind: "xsd:string", lang: ""}},
		{subject: "_:jon-org", predicate: "v:organisation-unit", object: {value: "Stewards", kind: "xsd:string", lang: ""}},
		
		{subject: "got:Sandor_Clegane", predicate: "v:fn", object: {value: "Sandor Clegane", kind: "xsd:string", lang: ""}},
		{subject: "got:Sandor_Clegane", predicate: "v:nickname", object: {value: "The Hound", kind: "xsd:string", lang: ""}}
		]);
	ret store;
}
