import io;
import io::writer_util;
import test_data::*;
import test_helpers::*;

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
	assert store.namespaces[5u] == {prefix: "got", path: "http://awoiaf.westeros.org/index.php/"};
	assert store.namespaces[7u] == {prefix: "foo", path: "http://www.whatever.org/"};
	let entries = store.subjects.get({nindex: 5u, name: "Eddard_Stark"});
	
	let entry = entries.data[2u];
	//io::println(#fmt["entry = %?", entry]);
	assert entry.predicate == {nindex: 7u, name: "child"};
	assert entry.object == ireference({nindex: 5u, name: "Jon_Snow"});
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

#[test]
fn trivial_bgp() 
{
	let store = create_store([]);
	let group1 = [];
	let group2 = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let expected = group2;
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn identical_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let group2 = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let expected = group2;
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn disjoint1_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let group2 = [
		[{name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	let expected = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn disjoint2_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}, {name: "job", value: ityped("cowboy", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "job", value: ityped("muckraker", {nindex: 2u, name: "string"})}]
	];
	let group2 = [
		[{name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	let expected = [
		[
			{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})},
			{name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})},
			{name: "job", value: ityped("muckraker", {nindex: 2u, name: "string"})},
			{name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}
		],
		[
			{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})},
			{name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})},
			{name: "job", value: ityped("muckraker", {nindex: 2u, name: "string"})},
			{name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}
		],
		[
			{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})},
			{name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})},
			{name: "job", value: ityped("cowboy", {nindex: 2u, name: "string"})},
			{name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}
		],
		[
			{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})},
			{name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})},
			{name: "job", value: ityped("cowboy", {nindex: 2u, name: "string"})},
			{name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}
		]
	];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn asymmetric_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "age", value: ityped("33", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let group2 = [
		[{name: "age", value: ityped("88", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	let expected = [
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn symmetric_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "age", value: ityped("33", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("25", {nindex: 2u, name: "integer"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}]
	];
	let group2 = [
		[{name: "age", value: ityped("88", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	let expected = [
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn path_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})}],
		[{name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})}],
		[{name: "name", value: ityped("George", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("ggg", {nindex: 2u, name: "string"})}]
	];
	let group2 = [
		[{name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}],
		[{name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("88", {nindex: 2u, name: "integer"})}],
		[{name: "id", value: ityped("zzz", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("38", {nindex: 2u, name: "integer"})}]
	];
	let expected = [
		[{name: "age", value: ityped("88", {nindex: 2u, name: "integer"})}, {name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})}, {name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}],
		[{name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}, {name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})}, {name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}]
	];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}

#[test]
fn incompatible_bgp() 
{
	let store = create_store([]);
	let group1 = [
		[{name: "name", value: ityped("Bob", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("bbb", {nindex: 2u, name: "string"})}],
		[{name: "name", value: ityped("Ted", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("ttt", {nindex: 2u, name: "string"})}],
		[{name: "name", value: ityped("George", {nindex: 2u, name: "string"})}, {name: "id", value: ityped("ggg", {nindex: 2u, name: "string"})}]
	];
	let group2 = [
		[{name: "id", value: ityped("tyt", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("18", {nindex: 2u, name: "integer"})}],
		[{name: "id", value: ityped("bxb", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("88", {nindex: 2u, name: "integer"})}],
		[{name: "id", value: ityped("zzz", {nindex: 2u, name: "string"})}, {name: "age", value: ityped("38", {nindex: 2u, name: "integer"})}]
	];
	let expected = [];
	
	assert check_algebra(store, [group1, group2], expected);
	assert check_algebra(store, [group2, group1], expected);
}
