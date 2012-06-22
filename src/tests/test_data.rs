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
	store.add("got:Eddard_Stark", [
		make_str("v:fn", "Eddard Stark"),
		make_str("v:nickname", "Ned")
	]);
	ret store;
}

fn got_cast3() -> store
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "v", path: "http://www.w3.org/2006/vcard/ns#"}
		]);
	
	store.add("got:Eddard_Stark", [
		make_str("v:fn", "Eddard Stark"),
		make_str("v:nickname", "Ned"),
		make_str("v:honorific-prefix", "Lord")
	]);
	store.add_aggregate("got:Eddard_Stark", "v:org", "ned-org", [
		make_str("v:organisation-name", "Small Council"),
		make_str("v:organisation-unit", "Hand")
	]);
	
	store.add("got:Jon_Snow", [
		make_str("v:fn", "Jon Snow"),
		make_str("v:nickname", "Lord Snow")
	]);
	store.add_aggregate("got:Jon_Snow", "v:org", "jon-org", [
		make_str("v:organisation-name", "Night's Watch"),
		make_str("v:organisation-unit", "Stewards")
	]);
	
	store.add("got:Sandor_Clegane", [
		make_str("v:fn", "Sandor Clegane"),
		make_str("v:nickname", "The Hound")
	]);
	ret store;
}
