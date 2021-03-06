// Hard-coded triple stores used for unit testing. These use a few different schemas:
// 1) http://www.w3.org/TR/vcard-rdf/
// Profile for electronix business cards.
// 2) https://github.com/edumbill/doap/wiki
// Profile used to describe open source software projects.

pub fn got_cast1() -> Store
{
	let store = Store(~[
		Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		Namespace {prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"}
		], @HashMap());
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", @StringValue(~"Eddard Stark", ~"")),
		(~"v:nickname", @StringValue(~"Ned", ~""))
	]);
	return store;
}

pub fn got_cast3() -> Store
{
	let store = Store(~[
		Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		Namespace {prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"}
		], @HashMap());
	
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", @StringValue(~"Eddard Stark", ~"")),
		(~"v:nickname", @StringValue(~"Ned", ~"")),
		(~"v:honorific-prefix", @StringValue(~"Lord", ~""))
	]);
	store.add_aggregate(~"got:Eddard_Stark", ~"v:org", ~"ned-org", ~[
		(~"v:organisation-name", @StringValue(~"Small Council", ~"")),
		(~"v:organisation-unit", @StringValue(~"Hand", ~""))
	]);
	
	store.add(~"got:Jon_Snow", ~[
		(~"v:fn", @StringValue(~"Jon Snow", ~"")),
		(~"v:nickname", @StringValue(~"Lord Snow", ~"")),
		(~"v:pet", @StringValue(~"Ghost", ~""))
	]);
	store.add_aggregate(~"got:Jon_Snow", ~"v:org", ~"jon-org", ~[
		(~"v:organisation-name", @StringValue(~"Night's Watch", ~"")),
		(~"v:organisation-unit", @StringValue(~"Stewards", ~""))
	]);
	
	store.add(~"got:Sandor_Clegane", ~[
		(~"v:fn", @StringValue(~"Sandor Clegane", ~"")),
		(~"v:nickname", @StringValue(~"The Hound", ~""))
	]);
	return store;
}

pub fn animals() -> Store
{
	let store = Store(~[Namespace {prefix: ~"wiki", path: ~"http://en.wikipedia.org/wiki/"}], @HashMap());
	
	store.add(~"wiki:aardvark", ~[
		(~"wiki:phylum", @StringValue(~"chordata", ~"")),
		(~"wiki:class", @StringValue(~"mammalia", ~"")),
		(~"wiki:family", @StringValue(~"orycteropodidae", ~"")),
	]);
		
	store.add(~"wiki:black_widow", ~[
		(~"wiki:phylum", @StringValue(~"arthropoda", ~"")),
		(~"wiki:class", @StringValue(~"arachnida", ~"")),
		(~"wiki:family", @StringValue(~"theridiidae", ~"")),
	]);
		
	store.add(~"wiki:firefly", ~[
		(~"wiki:phylum", @StringValue(~"arthropoda", ~"")),
		(~"wiki:class", @StringValue(~"insecta", ~"")),
		(~"wiki:family", @StringValue(~"lampyridae", ~"")),
	]);
		
	store.add(~"wiki:giraffe", ~[
		(~"wiki:phylum", @StringValue(~"chordata", ~"")),
		(~"wiki:class", @StringValue(~"mammalia", ~"")),
		(~"wiki:family", @StringValue(~"giraffidae", ~"")),
		(~"wiki:habitat", @StringValue(~"savannah", ~"")),
	]);
		
	store.add(~"wiki:grizzly", ~[
		(~"wiki:phylum", @StringValue(~"chordata", ~"")),
		(~"wiki:class", @StringValue(~"mammalia", ~"")),
		(~"wiki:family", @StringValue(~"ursidae", ~"")),
	]);
		
	store.add(~"wiki:salmon", ~[
		(~"wiki:phylum", @StringValue(~"chordata", ~"")),
		(~"wiki:class", @StringValue(~"actinopterygii", ~"")),
		(~"wiki:family", @StringValue(~"salmonidae", ~"")),
	]);
	
	return store;
}
