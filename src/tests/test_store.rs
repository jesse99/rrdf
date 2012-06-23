import io;
import io::writer_util;
import std::map::*;
import test_data::*;
import test_helpers::*;

#[test]
fn to_strs()
{
	let obj = {value: "some long url", kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""};
	assert check_strs(obj.to_str(), "some long url");
	
	let obj = {value: "12", kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""};
	assert check_strs(obj.to_str(), "\"12\"^^http://www.w3.org/2001/XMLSchema#integer");
	
	let obj = {value: "12", kind: "http://www.w3.org/2001/XMLSchema#string", lang: "en"};
	assert check_strs(obj.to_str(), "\"12\"@en");
	
	let obj = {value: "12", kind: "http://www.w3.org/2001/XMLSchema#string", lang: ""};
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
		make_triple_str(store, "got:Eddard_Stark", "v:fn", "Eddard Stark"),
		make_triple_str(store, "got:Eddard_Stark", "v:nickname", "Ned")
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
		
	store.add("got:Eddard_Stark", [
		("v:fn", create_str("Eddard Stark")),
		("v:nickname", create_str("Ned")),
		("foo:child", create_uri("got:Jon_Snow"))
	]);
	store.add("got:Jon_Snow", [
		("v:fn", create_str("Jon Snow"))
	]);
	
	let mut actual = [];
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	// The store will have full URIs (make_triple_* does the expansion as well).
	let expected = [
		make_triple_str(store, "got:Eddard_Stark", "v:fn", "Eddard Stark"),
		make_triple_str(store, "got:Eddard_Stark", "v:nickname", "Ned"),
		make_triple_uri(store, "got:Eddard_Stark", "foo:child", "got:Jon_Snow"),
		make_triple_str(store, "got:Jon_Snow", "v:fn", "Jon Snow")
	];
	
	assert check_triples(actual, expected);
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
		make_triple_str(store, "got:Eddard_Stark", "v:fn", "Eddard Stark"),
		make_triple_str(store, "got:Eddard_Stark", "v:nickname", "Ned"),
		make_triple_str(store, "got:Eddard_Stark", "v:honorific-prefix", "Lord"),
		make_triple_blank(store, "got:Eddard_Stark", "v:org", "ned-org-0"),
		
		make_triple_str(store, "got:Jon_Snow", "v:fn", "Jon Snow"),
		make_triple_str(store, "got:Jon_Snow", "v:nickname", "Lord Snow"),
		make_triple_blank(store, "got:Jon_Snow", "v:org", "jon-org-1"),
		
		make_triple_str(store, "got:Sandor_Clegane", "v:fn", "Sandor Clegane"),
		make_triple_str(store, "got:Sandor_Clegane", "v:nickname", "The Hound"),

		make_triple_str(store, "{jon-org-1}", "v:organisation-name", "Night's Watch"),
		make_triple_str(store, "{jon-org-1}", "v:organisation-unit", "Stewards"),
		make_triple_str(store, "{ned-org-0}", "v:organisation-name", "Small Council"),
		make_triple_str(store, "{ned-org-0}", "v:organisation-unit", "Hand")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn container() 
{
	let store = create_store([{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"}]);
	store.add_alt("got:places", [create_uri("got:The_Wall"), create_uri("got:Winterfell")]);
	
	let mut actual = [];
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	let expected = [
		make_triple_blank(store, "got:places", "rdf:Alt", "places-items-0"),
		make_triple_uri(store, "{places-items-0}", "rdf:_1", "got:The_Wall"),
		make_triple_uri(store, "{places-items-0}", "rdf:_2", "got:Winterfell")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn list0() 
{
	let store = create_store([{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"}]);
	store.add_list("got:westeros", "got:cities", []);
	
	let mut actual = [];
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	let expected = [
		make_triple_blank(store, "got:westeros", "got:cities", "cities-0"),
		make_triple_uri(store, "{cities-0}", "rdf:rest", "rdf:nil")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn list1() 
{
	let store = create_store([{prefix: "got", path: "http://awoiaf.westeros.org/index.php/"}]);
	store.add_list("got:westeros", "got:cities", [create_str("Lanisport")]);
	
	let mut actual = [];
	for each_triple(store)
	{|triple|
		vec::push(actual, triple);
	};
	
	let expected = [
		make_triple_blank(store, "got:westeros", "got:cities", "cities-0"),
		
		make_triple_str(store, "{cities-0}", "rdf:first", "Lanisport"),
		make_triple_blank(store, "{cities-0}", "rdf:rest", "cities-1"),
		
		make_triple_uri(store, "{cities-1}", "rdf:rest", "rdf:nil")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn trivial_bgp() 
{
	let group1 = [];
	let group2 = [
		[("age", create_int(25))],
		[("age", create_int(18))]
	];
	let expected = group2;
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn identical_bgp()
{
	let group1 = [
		[("age", create_int(25))],		// TODO: use some fancy regex, remember \1 is atually $1
		[("age", create_int(18))]
	];
	let group2 = [
		[("age", create_int(25))],
		[("age", create_int(18))]
	];
	let expected = group2;
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn disjoint1_bgp()
{
	let group1 = [
		[("age", create_int(25))],
		[("age", create_int(18))]
	];
	let group2 = [
		[("name", create_str("Bob"))],
		[("name", create_str("Ted"))]
	];
	let expected = [
		[("age", create_int(18)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Ted"))],
		[("age", create_int(25)), ("name", create_str("Bob"))],
		[("age", create_int(25)), ("name", create_str("Ted"))]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn disjoint2_bgp() 
{
	let group1 = [
		[("age", create_int(25)), ("job", create_str("cowboy"))],
		[("age", create_int(18)), ("job", create_str("muckraker"))]
	];
	let group2 = [
		[("id", create_str("bbb")), ("name", create_str("Bob"))],
		[("id", create_str("ttt")), ("name", create_str("Ted"))]
	];
	let expected = [
		[("age", create_int(18)), ("id", create_str("bbb")), ("job", create_str("muckraker")), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("id", create_str("ttt")), ("job", create_str("muckraker")), ("name", create_str("Ted"))],
		[("age", create_int(25)), ("id", create_str("bbb")), ("job", create_str("cowboy")), ("name", create_str("Bob"))],
		[("age", create_int(25)), ("id", create_str("ttt")), ("job", create_str("cowboy")), ("name", create_str("Ted"))]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn asymmetric_bgp() 
{
	let group1 = [
		[("age", create_int(33))],
		[("age", create_int(25))],
		[("age", create_int(18))]
	];
	let group2 = [
		[("age", create_int(88)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Ted"))]
	];
	
	let expected = [
		[("age", create_int(18)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Ted"))]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn symmetric_bgp() 
{
	let group1 = [
		[("age", create_int(33))],
		[("age", create_int(25))],
		[("age", create_int(18))]
	];
	let group2 = [
		[("age", create_int(88)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Ted"))]
	];
	let expected = [
		[("age", create_int(18)), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("name", create_str("Ted"))]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn path_bgp() 
{
	let group1 = [
		[("name", create_str("Bob")), ("id", create_str("bbb"))],
		[("name", create_str("Ted")), ("id", create_str("ttt"))],
		[("name", create_str("George")), ("id", create_str("ggg"))]
	];
	let group2 = [
		[("id", create_str("ttt")), ("age", create_int(18))],
		[("id", create_str("bbb")), ("age", create_int(88))],
		[("id", create_str("zzz")), ("age", create_int(38))]
	];
	let expected = [
		[("age", create_int(88)), ("id", create_str("bbb")), ("name", create_str("Bob"))],
		[("age", create_int(18)), ("id", create_str("ttt")), ("name", create_str("Ted"))]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn incompatible_bgp() 
{
	let group1 = [
		[("name", create_str("Bob")), ("id", create_str("bbb"))],
		[("name", create_str("Ted")), ("id", create_str("ttt"))],
		[("name", create_str("George")), ("id", create_str("ggg"))]
	];
	let group2 = [
		[("id", create_str("tyt")), ("age", create_int(18))],
		[("id", create_str("bxb")), ("age", create_int(88))],
		[("id", create_str("zzz")), ("age", create_int(38))]
	];
	let expected = [];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}
