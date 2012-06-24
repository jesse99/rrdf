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
		("v:fn", create_str("Eddard Stark")),
		("v:nickname", create_str("Ned"))
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
		("v:fn", create_str("Eddard Stark")),
		("v:nickname", create_str("Ned")),
		("v:honorific-prefix", create_str("Lord"))
	]);
	store.add_aggregate("got:Eddard_Stark", "v:org", "ned-org", [
		("v:organisation-name", create_str("Small Council")),
		("v:organisation-unit", create_str("Hand"))
	]);
	
	store.add("got:Jon_Snow", [
		("v:fn", create_str("Jon Snow")),
		("v:nickname", create_str("Lord Snow")),
		("v:pet", create_str("Ghost"))
	]);
	store.add_aggregate("got:Jon_Snow", "v:org", "jon-org", [
		("v:organisation-name", create_str("Night's Watch")),
		("v:organisation-unit", create_str("Stewards"))
	]);
	
	store.add("got:Sandor_Clegane", [
		("v:fn", create_str("Sandor Clegane")),
		("v:nickname", create_str("The Hound"))
	]);
	ret store;
}
