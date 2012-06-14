import io;
import io::writer_util;
import test_data::*;

fn check_strs(actual: str, expected: str) -> bool
{
	if actual != expected
	{
		io::stderr().write_line(#fmt["Found '%s', but expected '%s'", actual, expected]);
		ret false;
	}
	ret true;
}

fn check_triples(actual: [triple], expected: [triple]) -> bool
{
	fn dump_triples(actual: [triple])
	{
		io::stderr().write_line("Actual triples:");
		for vec::each(actual)
		{|triple|
			io::stderr().write_line(#fmt["   %s", triple.to_str()]);
		};
	}
	
	let actual = std::sort::merge_sort({|x, y| x.subject <= y.subject}, actual);
	let expected = std::sort::merge_sort({|x, y| x.subject <= y.subject}, expected);
	
	if vec::len(actual) != vec::len(expected)
	{
		io::stderr().write_line(#fmt["Actual length is %?, but expected %?", vec::len(actual), vec::len(expected)]);
		dump_triples(actual);
		ret false;
	}
	
	for vec::eachi(actual)
	{|i, atriple|
		let etriple = expected[i];
		
		if atriple.subject != etriple.subject
		{
			io::stderr().write_line(#fmt["Subject #%? is %?, but expected %?", i, atriple.subject, etriple.subject]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.predicate != etriple.predicate
		{
			io::stderr().write_line(#fmt["Predicate #%? is %?, but expected %?", i, atriple.predicate, etriple.predicate]);
			dump_triples(actual);
			ret false;
		}
		
		if atriple.object != etriple.object
		{
			io::stderr().write_line(#fmt["Object #%? is %?, but expected %?", i, atriple.object.to_str(), etriple.object.to_str()]);
			dump_triples(actual);
			ret false;
		}
	}
	
	ret true;
}

#[test]
fn to_strs()
{
	let obj = {value: "some long url", kind: "xsd:anyURI", lang: ""};
	assert check_strs(obj.to_str(), "some long url");
	
	let obj = {value: "12", kind: "xsd:integer", lang: ""};
	assert check_strs(obj.to_str(), "\"12\"^^xsd:integer");
	
	let obj = {value: "12", kind: "xsd:string", lang: "en"};
	assert check_strs(obj.to_str(), "\"12\"@en");
	
	let obj = {value: "12", kind: "xsd:string", lang: ""};
	assert check_strs(obj.to_str(), "\"12\"");
}

#[test]
fn iteration() 
{
	let store = got_cast1();
	let mut actual = [];
	
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	let expected = [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: {value: "Eddard Stark", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: {value: "Ned", kind: "xsd:string", lang: ""}}
		];
	assert check_triples(actual, expected);
}

#[test]
fn references() 
{
	let store = create_store([
		{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"},
		{prefix: "v", path: "http://www.w3.org/2006/vcard/ns#"},
		{prefix: "foo", path: "http://www.whatever.org/"}
		]);
	
	add_triples(store, [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: {value: "Eddard Stark", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: {value: "Ned", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "foo:child", object: {value: "got:Jon_Snow", kind: "xsd:anyURI", lang: ""}},
		{subject: "got:Jon_Snow", predicate: "v:fn", object: {value: "Jon Snow", kind: "xsd:string", lang: ""}}
		]);
	
	let mut actual = [];
	
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	// When we round-trip we should wind up with references again.
	let expected = [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: {value: "Eddard Stark", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: {value: "Ned", kind: "xsd:string", lang: ""}},
		{subject: "got:Eddard_Stark", predicate: "foo:child", object: {value: "got:Jon_Snow", kind: "xsd:anyURI", lang: ""}},
		{subject: "got:Jon_Snow", predicate: "v:fn", object: {value: "Jon Snow", kind: "xsd:string", lang: ""}}
		];
	assert check_triples(actual, expected);
	
	// But internally references are stored as qrefs.
	assert store.namespaces[3u] == {prefix: "got", path: "http://awoiaf.westeros.org/index.php/"};
	assert store.namespaces[5u] == {prefix: "foo", path: "http://www.whatever.org/"};
	let entries = store.subjects.get({nindex: 3u, name: "Eddard_Stark"});
	
	let entry = entries.data[2u];
	//io::println(#fmt["entry = %?", entry]);
	assert entry.predicate == {nindex: 5u, name: "child"};
	assert entry.object == ireference({nindex: 3u, name: "Jon_Snow"});
}

#[test]
fn blank_nodes() 
{
	let store = got_cast3();
	let mut actual = [];
	
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	let expected = [
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
		];
	assert check_triples(actual, expected);
}
