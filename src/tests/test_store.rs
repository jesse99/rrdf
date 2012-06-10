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
	let obj = reference("some log url");
	assert check_strs(obj.to_str(), "some log url");
	
	let obj = qref({nindex: 2u, name: "integer"});
	assert check_strs(obj.to_str(), "2:integer");
	
	let obj = typed_literal("12", "xsd:integer");
	assert check_strs(obj.to_str(), "\"12\"^^xsd:integer");
	
	let obj = plain_literal("12", "en");
	assert check_strs(obj.to_str(), "\"12\"@en");
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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: typed_literal("Eddard Stark", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: typed_literal("Ned", "xsd:string")}
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
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: typed_literal("Eddard Stark", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: typed_literal("Ned", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "foo:child", object: reference("got:Jon_Snow")},
		{subject: "got:Jon_Snow", predicate: "v:fn", object: typed_literal("Jon Snow", "xsd:string")}
		]);
	
	let mut actual = [];
	
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	// When we round-trip we should wind up with references again.
	let expected = [
		{subject: "got:Eddard_Stark", predicate: "v:fn", object: typed_literal("Eddard Stark", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "v:nickname", object: typed_literal("Ned", "xsd:string")},
		{subject: "got:Eddard_Stark", predicate: "foo:child", object: reference("got:Jon_Snow")},
		{subject: "got:Jon_Snow", predicate: "v:fn", object: typed_literal("Jon Snow", "xsd:string")}
		];
	assert check_triples(actual, expected);
	
	// But internally references are stored as qrefs.
	assert store.namespaces[2u] == {prefix: "got", path: "http://awoiaf.westeros.org/index.php/"};
	assert store.namespaces[4u] == {prefix: "foo", path: "http://www.whatever.org/"};
	let entries = store.subjects.get({nindex: 2u, name: "Eddard_Stark"});
	
	let entry = entries.data[2u];
	io::println(#fmt["entry = %?", entry]);
	assert entry.predicate == {nindex: 4u, name: "child"};
	assert entry.object == qref({nindex: 2u, name: "Jon_Snow"});
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
		];
	assert check_triples(actual, expected);
}

// TODO:
// query
// remove
// clean
// add_store
