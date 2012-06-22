import io;
import io::writer_util;
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
		make_str("v:fn", "Eddard Stark"),
		make_str("v:nickname", "Ned"),
		make_uri("foo:child", "got:Jon_Snow")
	]);
	store.add("got:Jon_Snow", [
		make_str("v:fn", "Jon Snow")
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
fn trivial_bgp() 
{
	let group1 = [];
	let group2 = [
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let expected = group2;
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn identical_bgp() 
{
	let group1 = [
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let group2 = [
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let expected = group2;
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn disjoint1_bgp() 
{
	let group1 = [
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let group2 = [
		[bind_str("name", "Bob")],
		[bind_str("name", "Ted")]
	];
	let expected = [
		[bind_int("age", 18), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Ted")],
		[bind_int("age", 25), bind_str("name", "Bob")],
		[bind_int("age", 25), bind_str("name", "Ted")]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn disjoint2_bgp() 
{
	let group1 = [
		[bind_int("age", 25), bind_str("job", "cowboy")],
		[bind_int("age", 18), bind_str("job", "muckraker")]
	];
	let group2 = [
		[bind_str("id", "bbb"), bind_str("name", "Bob")],
		[bind_str("id", "ttt"), bind_str("name", "Ted")]
	];
	let expected = [
		[bind_int("age", 18), bind_str("id", "bbb"), bind_str("job", "muckraker"), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("id", "ttt"), bind_str("job", "muckraker"), bind_str("name", "Ted")],
		[bind_int("age", 25), bind_str("id", "bbb"), bind_str("job", "cowboy"), bind_str("name", "Bob")],
		[bind_int("age", 25), bind_str("id", "ttt"), bind_str("job", "cowboy"), bind_str("name", "Ted")]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn asymmetric_bgp() 
{
	let group1 = [
		[bind_int("age", 33)],
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let group2 = [
		[bind_int("age", 88), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Ted")]
	];
	let expected = [
		[bind_int("age", 18), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Ted")]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn symmetric_bgp() 
{
	let group1 = [
		[bind_int("age", 33)],
		[bind_int("age", 25)],
		[bind_int("age", 18)]
	];
	let group2 = [
		[bind_int("age", 88), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Ted")]
	];
	let expected = [
		[bind_int("age", 18), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("name", "Ted")]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn path_bgp() 
{
	let group1 = [
		[bind_str("name", "Bob"), bind_str("id", "bbb")],
		[bind_str("name", "Ted"), bind_str("id", "ttt")],
		[bind_str("name", "George"), bind_str("id", "ggg")]
	];
	let group2 = [
		[bind_str("id", "ttt"), bind_int("age", 18)],
		[bind_str("id", "bbb"), bind_int("age", 88)],
		[bind_str("id", "zzz"), bind_int("age", 38)]
	];
	let expected = [
		[bind_int("age", 88), bind_str("id", "bbb"), bind_str("name", "Bob")],
		[bind_int("age", 18), bind_str("id", "ttt"), bind_str("name", "Ted")]
	];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}

#[test]
fn incompatible_bgp() 
{
	let group1 = [
		[bind_str("name", "Bob"), bind_str("id", "bbb")],
		[bind_str("name", "Ted"), bind_str("id", "ttt")],
		[bind_str("name", "George"), bind_str("id", "ggg")]
	];
	let group2 = [
		[bind_str("id", "tyt"), bind_int("age", 18)],
		[bind_str("id", "bxb"), bind_int("age", 88)],
		[bind_str("id", "zzz"), bind_int("age", 38)]
	];
	let expected = [];
	
	assert check_bgp([group1, group2], expected);
	assert check_bgp([group2, group1], expected);
}
