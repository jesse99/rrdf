// Hard-coded triple stores used for unit testing. These use a few different schemas:
// 1) http://www.w3.org/TR/vcard-rdf/
// Profile for electronix business cards.
// 2) https://github.com/edumbill/doap/wiki
// Profile used to describe open source software projects.

fn got_cast1() -> store
{
	let store = create_store(~[
		{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		{prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"}
		], @std::map::str_hash());
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", string_value(~"Eddard Stark", ~"")),
		(~"v:nickname", string_value(~"Ned", ~""))
	]);
	ret store;
}

fn got_cast3() -> store
{
	let store = create_store(~[
		{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		{prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"}
		], @std::map::str_hash());
	
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", string_value(~"Eddard Stark", ~"")),
		(~"v:nickname", string_value(~"Ned", ~"")),
		(~"v:honorific-prefix", string_value(~"Lord", ~""))
	]);
	store.add_aggregate(~"got:Eddard_Stark", ~"v:org", ~"ned-org", ~[
		(~"v:organisation-name", string_value(~"Small Council", ~"")),
		(~"v:organisation-unit", string_value(~"Hand", ~""))
	]);
	
	store.add(~"got:Jon_Snow", ~[
		(~"v:fn", string_value(~"Jon Snow", ~"")),
		(~"v:nickname", string_value(~"Lord Snow", ~"")),
		(~"v:pet", string_value(~"Ghost", ~""))
	]);
	store.add_aggregate(~"got:Jon_Snow", ~"v:org", ~"jon-org", ~[
		(~"v:organisation-name", string_value(~"Night's Watch", ~"")),
		(~"v:organisation-unit", string_value(~"Stewards", ~""))
	]);
	
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:fn", string_value(~"Sandor Clegane", ~"")),
		(~"v:nickname", string_value(~"The Hound", ~""))
	]);
	ret store;
}

fn animals() -> store
{
	let store = create_store(~[{prefix: ~"wiki", path: ~"http://en.wikipedia.org/wiki/"}], @std::map::str_hash());
	
	store.add(~"wiki:aardvark", ~[
		(~"wiki:phylum", string_value(~"chordata", ~"")),
		(~"wiki:class", string_value(~"mammalia", ~"")),
		(~"wiki:family", string_value(~"orycteropodidae", ~"")),
	]);
		
	store.add(~"wiki:black_widow", ~[
		(~"wiki:phylum", string_value(~"arthropoda", ~"")),
		(~"wiki:class", string_value(~"arachnida", ~"")),
		(~"wiki:family", string_value(~"theridiidae", ~"")),
	]);
		
	store.add(~"wiki:firefly", ~[
		(~"wiki:phylum", string_value(~"arthropoda", ~"")),
		(~"wiki:class", string_value(~"insecta", ~"")),
		(~"wiki:family", string_value(~"lampyridae", ~"")),
	]);
		
	store.add(~"wiki:giraffe", ~[
		(~"wiki:phylum", string_value(~"chordata", ~"")),
		(~"wiki:class", string_value(~"mammalia", ~"")),
		(~"wiki:family", string_value(~"giraffidae", ~"")),
		(~"wiki:habitat", string_value(~"savannah", ~"")),
	]);
		
	store.add(~"wiki:grizzly", ~[
		(~"wiki:phylum", string_value(~"chordata", ~"")),
		(~"wiki:class", string_value(~"mammalia", ~"")),
		(~"wiki:family", string_value(~"ursidae", ~"")),
	]);
		
	store.add(~"wiki:salmon", ~[
		(~"wiki:phylum", string_value(~"chordata", ~"")),
		(~"wiki:class", string_value(~"actinopterygii", ~"")),
		(~"wiki:family", string_value(~"salmonidae", ~"")),
	]);
	
	ret store;
}
