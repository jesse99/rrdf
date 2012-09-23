use io::WriterUtil;
use std::map::*;
use object::*;
use solution::*;
use test_data::*;
use test_helpers::*;

#[test]
fn to_strs()
{
	let obj = literal_to_object(@~"some long url", @~"http://www.w3.org/2001/XMLSchema#anyURI", @~"");
	assert check_strs(obj.to_str(), ~"<some long url>");
	
	let obj = literal_to_object(@~"12", @~"http://www.w3.org/2001/XMLSchema#integer", @~"");
	assert check_strs(obj.to_str(), ~"12");
	
	let obj = literal_to_object(@~"12", @~"http://www.w3.org/2001/XMLSchema#string", @~"en");
	assert check_strs(obj.to_str(), ~"\"12\"@en");
	
	let obj = literal_to_object(@~"12", @~"http://www.w3.org/2001/XMLSchema#string", @~"");
	assert check_strs(obj.to_str(), ~"\"12\"");
}

#[test]
fn iteration()
{
	let store = got_cast1();
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:fn", ~"Eddard Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:nickname", ~"Ned")
	];
	assert check_triples(actual, expected);
}

#[test]
fn references()
{
	let store = Store(~[
		{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		{prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"},
		{prefix: ~"foo", path: ~"http://www.whatever.org/"}
		], &std::map::HashMap());
		
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", StringValue(~"Eddard Stark", ~"")),
		(~"v:nickname", StringValue(~"Ned", ~"")),
		(~"foo:child", IriValue(~"got:Jon_Snow"))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:fn", StringValue(~"Jon Snow", ~""))
	]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	// The store will have full URIs (make_triple_* does the expansion as well).
	let expected = ~[
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:fn", ~"Eddard Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:nickname", ~"Ned"),
		make_triple_uri(&store, ~"got:Eddard_Stark", ~"foo:child", ~"got:Jon_Snow"),
		make_triple_str(&store, ~"got:Jon_Snow", ~"v:fn", ~"Jon Snow")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn blank_nodes() 
{
	let store = got_cast3();
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_str(&store, ~"_:jon-org-1", ~"v:organisation-name", ~"Night's Watch"),
		make_triple_str(&store, ~"_:jon-org-1", ~"v:organisation-unit", ~"Stewards"),
		make_triple_str(&store, ~"_:ned-org-0", ~"v:organisation-name", ~"Small Council"),
		make_triple_str(&store, ~"_:ned-org-0", ~"v:organisation-unit", ~"Hand"),
		
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:fn", ~"Eddard Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:nickname", ~"Ned"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:honorific-prefix", ~"Lord"),
		make_triple_blank(&store, ~"got:Eddard_Stark", ~"v:org", ~"ned-org-0"),
		
		make_triple_str(&store, ~"got:Jon_Snow", ~"v:fn", ~"Jon Snow"),
		make_triple_str(&store, ~"got:Jon_Snow", ~"v:nickname", ~"Lord Snow"),
		make_triple_str(&store, ~"got:Jon_Snow", ~"v:pet", ~"Ghost"),
		make_triple_blank(&store, ~"got:Jon_Snow", ~"v:org", ~"jon-org-1"),
		
		make_triple_str(&store, ~"got:Sandor_Clegane", ~"v:fn", ~"Sandor Clegane"),
		make_triple_str(&store, ~"got:Sandor_Clegane", ~"v:nickname", ~"The Hound"),
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn container() 
{
	let store = Store(~[{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &std::map::HashMap());
	store.add_alt(~"got:places", ~[IriValue(~"got:The_Wall"), IriValue(~"got:Winterfell")]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_blank(&store, ~"got:places", ~"rdf:Alt", ~"places-items-0"),
		make_triple_uri(&store, ~"_:places-items-0", ~"rdf:_1", ~"got:The_Wall"),
		make_triple_uri(&store, ~"_:places-items-0", ~"rdf:_2", ~"got:Winterfell")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn list0() 
{
	let store = Store(~[{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &std::map::HashMap());
	store.add_list(~"got:westeros", ~"got:cities", ~[]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_blank(&store, ~"got:westeros", ~"got:cities", ~"cities-0"),
		make_triple_uri(&store, ~"_:cities-0", ~"rdf:rest", ~"rdf:nil")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn list1() 
{
	let store = Store(~[{prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &std::map::HashMap());
	store.add_list(~"got:westeros", ~"got:cities", ~[StringValue(~"Lanisport", ~"")]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_blank(&store, ~"got:westeros", ~"got:cities", ~"cities-0"),
		
		make_triple_str(&store, ~"_:cities-0", ~"rdf:first", ~"Lanisport"),
		make_triple_blank(&store, ~"_:cities-0", ~"rdf:rest", ~"cities-1"),
		
		make_triple_uri(&store, ~"_:cities-1", ~"rdf:rest", ~"rdf:nil")
	];
	
	assert check_triples(actual, expected);
}

#[test]
fn reify() 
{
	let store = got_cast1();
	store.add_reify(~"got:Eddard_Stark", ~"got:wife", IriValue(~"got:Caitlyn_Stark"));
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_uri(&store, ~"_:wife-0", ~"rdf:type", ~"rdf:Statement"),
		make_triple_uri(&store, ~"_:wife-0", ~"rdf:subject", ~"got:Eddard_Stark"),
		make_triple_uri(&store, ~"_:wife-0", ~"rdf:predicate", ~"got:wife"),
		make_triple_uri(&store, ~"_:wife-0", ~"rdf:object", ~"got:Caitlyn_Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:fn", ~"Eddard Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:nickname", ~"Ned"),
	];
	assert check_triples(actual, expected);
}

#[test]
fn replace() 
{
	let store = got_cast1();
	store.replace_triple(~[], {subject: ~"got:Eddard_Stark", predicate: ~"v:nickname", object: StringValue(~"Ned the Dead", ~"")});
	store.replace_triple(~[], {subject: ~"got:Arya", predicate: ~"v:nickname", object: StringValue(~"Underfoot", ~"")});
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(actual, *triple);
	};
	
	let expected = ~[
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:fn", ~"Eddard Stark"),
		make_triple_str(&store, ~"got:Eddard_Stark", ~"v:nickname", ~"Ned the Dead"),
		make_triple_str(&store, ~"got:Arya", ~"v:nickname", ~"Underfoot"),
	];
	assert check_triples(actual, expected);
}

#[test]
fn trivial_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(25i64))],
		~[(~"age", IntValue(18i64))]
	]};
	
	assert check_bgp(~[group1, group2], Solution {namespaces: ~[], rows: ~[]});
	assert check_bgp(~[group2, group1], Solution {namespaces: ~[], rows: ~[]});
}

#[test]
fn identical_bgp()
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(25i64))],		// TODO: use some fancy regex, remember \1 is atually $1
		~[(~"age", IntValue(18i64))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(25i64))],
		~[(~"age", IntValue(18i64))]
	]};
	let expected = group2;
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn disjoint1_bgp()
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(25i64))],
		~[(~"age", IntValue(18i64))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Bob", ~""))],
		~[(~"name", StringValue(~"Ted", ~""))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Ted", ~""))],
		~[(~"age", IntValue(25i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(25i64)), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn disjoint2_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(25i64)), (~"job", StringValue(~"cowboy", ~""))],
		~[(~"age", IntValue(18i64)), (~"job", StringValue(~"muckraker", ~""))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"id", StringValue(~"bbb", ~"")), (~"name", StringValue(~"Bob", ~""))],
		~[(~"id", StringValue(~"ttt", ~"")), (~"name", StringValue(~"Ted", ~""))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(18i64)), (~"id", StringValue(~"bbb", ~"")), (~"job", StringValue(~"muckraker", ~"")), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"id", StringValue(~"ttt", ~"")), (~"job", StringValue(~"muckraker", ~"")), (~"name", StringValue(~"Ted", ~""))],
		~[(~"age", IntValue(25i64)), (~"id", StringValue(~"bbb", ~"")), (~"job", StringValue(~"cowboy", ~"")), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(25i64)), (~"id", StringValue(~"ttt", ~"")), (~"job", StringValue(~"cowboy", ~"")), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn asymmetric_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(33i64))],
		~[(~"age", IntValue(25i64))],
		~[(~"age", IntValue(18i64))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(88i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn symmetric_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(33i64))],
		~[(~"age", IntValue(25i64))],
		~[(~"age", IntValue(18i64))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(88i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Ted", ~""))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn path_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Bob", ~"")), (~"id", StringValue(~"bbb", ~""))],
		~[(~"name", StringValue(~"Ted", ~"")), (~"id", StringValue(~"ttt", ~""))],
		~[(~"name", StringValue(~"George", ~"")), (~"id", StringValue(~"ggg", ~""))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"id", StringValue(~"ttt", ~"")), (~"age", IntValue(18i64))],
		~[(~"id", StringValue(~"bbb", ~"")), (~"age", IntValue(88i64))],
		~[(~"id", StringValue(~"zzz", ~"")), (~"age", IntValue(38i64))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[
		~[(~"age", IntValue(88i64)), (~"id", StringValue(~"bbb", ~"")), (~"name", StringValue(~"Bob", ~""))],
		~[(~"age", IntValue(18i64)), (~"id", StringValue(~"ttt", ~"")), (~"name", StringValue(~"Ted", ~""))]
	]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn incompatible_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Bob", ~"")), (~"id", StringValue(~"bbb", ~""))],
		~[(~"name", StringValue(~"Ted", ~"")), (~"id", StringValue(~"ttt", ~""))],
		~[(~"name", StringValue(~"George", ~"")), (~"id", StringValue(~"ggg", ~""))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"id", StringValue(~"tyt", ~"")), (~"age", IntValue(18i64))],
		~[(~"id", StringValue(~"bxb", ~"")), (~"age", IntValue(88i64))],
		~[(~"id", StringValue(~"zzz", ~"")), (~"age", IntValue(38i64))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn multiple_bgp() 
{
	let group1 = Solution {namespaces: ~[], rows: ~[
		~[(~"name", StringValue(~"Bob", ~"")), (~"id", StringValue(~"bbb", ~""))],
		~[(~"name", StringValue(~"Ted", ~"")), (~"id", StringValue(~"ttt", ~""))],
		~[(~"name", StringValue(~"George", ~"")), (~"id", StringValue(~"ggg", ~""))]
	]};
	let group2 = Solution {namespaces: ~[], rows: ~[
		~[(~"id", StringValue(~"tyt", ~"")), (~"age", IntValue(18i64))],
		~[(~"id", StringValue(~"bxb", ~"")), (~"age", IntValue(88i64))],
		~[(~"id", StringValue(~"zzz", ~"")), (~"age", IntValue(38i64))]
	]};
	let expected = Solution {namespaces: ~[], rows: ~[]};
	
	assert check_bgp(~[group1, group2], expected);
	assert check_bgp(~[group2, group1], expected);
}

#[test]
fn get_blanks() 
{
	let store = got_cast1();
	
	let f1 = get_blank_name(&store, ~"foo");
	let f2 = get_blank_name(&store, ~"foo");
	let f3 = get_blank_name(&store, ~"foo");
	
	assert f1 != f2;
	assert f2 != f3;
}
