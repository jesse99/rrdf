use io::WriterUtil;
use test_data::*;
use test_helpers::*;

#[test]
fn to_strs()
{
	let obj = literal_to_object("some long url", "http://www.w3.org/2001/XMLSchema#anyURI", "");
	assert check_strs(obj.to_str(), ~"<some long url>");
	
	let obj = literal_to_object("12", "http://www.w3.org/2001/XMLSchema#integer", "");
	assert check_strs(obj.to_str(), ~"12");
	
	let obj = literal_to_object("12", "http://www.w3.org/2001/XMLSchema#string", "en");
	assert check_strs(obj.to_str(), ~"\"12\"@en");
	
	let obj = literal_to_object("12", "http://www.w3.org/2001/XMLSchema#string", "");
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
		vec::push(&mut actual, copy *triple);
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
		Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"},
		Namespace {prefix: ~"v", path: ~"http://www.w3.org/2006/vcard/ns#"},
		Namespace {prefix: ~"foo", path: ~"http://www.whatever.org/"}
		], &HashMap());
		
	store.add(~"got:Eddard_Stark", ~[
		(~"v:fn", @StringValue(~"Eddard Stark", ~"")),
		(~"v:nickname", @StringValue(~"Ned", ~"")),
		(~"foo:child", @IriValue(~"got:Jon_Snow"))
	]);
	store.add(~"got:Jon_Snow", ~[
		(~"v:fn", @StringValue(~"Jon Snow", ~""))
	]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
		vec::push(&mut actual, copy *triple);
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
	let store = Store(~[Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &HashMap());
	store.add_alt(~"got:places", ~[@IriValue(~"got:The_Wall"), @IriValue(~"got:Winterfell")]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
	let store = Store(~[Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &HashMap());
	store.add_list(~"got:westeros", ~"got:cities", ~[]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
	let store = Store(~[Namespace {prefix: ~"got", path: ~"http://awoiaf.westeros.org/index.php/"}], &HashMap());
	store.add_list(~"got:westeros", ~"got:cities", ~[@StringValue(~"Lanisport", ~"")]);
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
	store.add_reify(~"got:Eddard_Stark", ~"got:wife", @IriValue(~"got:Caitlyn_Stark"));
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
	store.replace_triple(~[], {subject: ~"got:Eddard_Stark", predicate: ~"v:nickname", object: @StringValue(~"Ned the Dead", ~"")});
	store.replace_triple(~[], {subject: ~"got:Arya", predicate: ~"v:nickname", object: @StringValue(~"Underfoot", ~"")});
	
	let mut actual = ~[];
	for store.each
	|triple|
	{
		vec::push(&mut actual, copy *triple);
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
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[
		~[@IntValue(25i64)],
		~[@IntValue(18i64)]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[]});
	assert check_bgp(&store, ~[copy group2, copy group1], &Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[]});
}

#[test]
fn identical_bgp()
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[
		~[@IntValue(25i64)],		// TODO: use some fancy regex, remember \1 is atually $1
		~[@IntValue(18i64)]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age"], num_selected: 1, rows: ~[
		~[@IntValue(25i64)],
		~[@IntValue(18i64)]
	]};
	let expected = copy group2;
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn disjoint1_bgp()
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(25i64), @UnboundValue],
		~[@IntValue(18i64), @UnboundValue]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@UnboundValue, @StringValue(~"Bob", ~"")],
		~[@UnboundValue, @StringValue(~"Ted", ~"")]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(18i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Ted", ~"")],
		~[@IntValue(25i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(25i64), @StringValue(~"Ted", ~"")]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn disjoint2_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"job", ~"name"], num_selected: 4, rows: ~[
		~[@IntValue(25i64), @UnboundValue, @StringValue(~"cowboy", ~""), @UnboundValue],
		~[@IntValue(18i64), @UnboundValue, @StringValue(~"muckraker", ~""), @UnboundValue]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"job", ~"name"], num_selected: 4, rows: ~[
		~[@UnboundValue, @StringValue(~"bbb", ~""), @UnboundValue, @StringValue(~"Bob", ~"")],
		~[@UnboundValue, @StringValue(~"ttt", ~""), @UnboundValue, @StringValue(~"Ted", ~"")]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"job", ~"name"], num_selected: 4, rows: ~[
		~[@IntValue(18i64), @StringValue(~"bbb", ~""), @StringValue(~"muckraker", ~""), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"ttt", ~""), @StringValue(~"muckraker", ~""), @StringValue(~"Ted", ~"")],
		~[@IntValue(25i64), @StringValue(~"bbb", ~""), @StringValue(~"cowboy", ~""), @StringValue(~"Bob", ~"")],
		~[@IntValue(25i64), @StringValue(~"ttt", ~""), @StringValue(~"cowboy", ~""), @StringValue(~"Ted", ~"")]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn asymmetric_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(33i64), @UnboundValue],
		~[@IntValue(25i64), @UnboundValue],
		~[@IntValue(18i64), @UnboundValue]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(88i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Ted", ~"")]
	]};
	
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(18i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Ted", ~"")]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn symmetric_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(33i64), @UnboundValue],
		~[@IntValue(25i64), @UnboundValue],
		~[@IntValue(18i64), @UnboundValue]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(88i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Ted", ~"")]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(18i64), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"Ted", ~"")]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn path_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 3, rows: ~[
		~[@UnboundValue, @StringValue(~"bbb", ~""), @StringValue(~"Bob", ~"")],
		~[@UnboundValue, @StringValue(~"ttt", ~""), @StringValue(~"Ted", ~"")],
		~[@UnboundValue, @StringValue(~"ggg", ~""), @StringValue(~"George", ~"")]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 3, rows: ~[
		~[@IntValue(18i64), @StringValue(~"ttt", ~""), @UnboundValue],
		~[@IntValue(88i64), @StringValue(~"bbb", ~""), @UnboundValue],
		~[@IntValue(38i64), @StringValue(~"zzz", ~""), @UnboundValue]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 3, rows: ~[
		~[@IntValue(88i64), @StringValue(~"bbb", ~""), @StringValue(~"Bob", ~"")],
		~[@IntValue(18i64), @StringValue(~"ttt", ~""), @StringValue(~"Ted", ~"")]
	]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn incompatible_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 2, rows: ~[
		~[@UnboundValue, @StringValue(~"bbb", ~""), @StringValue(~"Bob", ~"")],
		~[@UnboundValue, @StringValue(~"ttt", ~""), @StringValue(~"Ted", ~"")],
		~[@UnboundValue, @StringValue(~"ggg", ~""), @StringValue(~"George", ~"")]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 2, rows: ~[
		~[@IntValue(18i64), @StringValue(~"tyt", ~""), @UnboundValue],
		~[@IntValue(88i64), @StringValue(~"bxb", ~""), @UnboundValue],
		~[@IntValue(38i64), @StringValue(~"zzz", ~""), @UnboundValue]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"age", ~"id", ~"name"], num_selected: 2, rows: ~[]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
}

#[test]
fn multiple_bgp() 
{
	let group1 = Solution {namespaces: ~[], bindings: ~[~"name", ~"id", ~"age"], num_selected: 2, rows: ~[
		~[@StringValue(~"Bob", ~""), @StringValue(~"bbb", ~""),  @UnboundValue],
		~[@StringValue(~"Ted", ~""), @StringValue(~"ttt", ~""),  @UnboundValue],
		~[@StringValue(~"George", ~""), @StringValue(~"ggg", ~""),  @UnboundValue]
	]};
	let group2 = Solution {namespaces: ~[], bindings: ~[~"name", ~"id", ~"age"], num_selected: 2, rows: ~[
		~[@UnboundValue, @StringValue(~"tyt", ~""), @IntValue(18i64)],
		~[@UnboundValue, @StringValue(~"bxb", ~""), @IntValue(88i64)],
		~[@UnboundValue, @StringValue(~"zzz", ~""), @IntValue(38i64)]
	]};
	let expected = Solution {namespaces: ~[], bindings: ~[~"name", ~"id", ~"age"], num_selected: 2, rows: ~[]};
	
	let store = Store(~[], &HashMap());
	assert check_bgp(&store, ~[copy group1, copy group2], &expected);
	assert check_bgp(&store, ~[copy group2, copy group1], &expected);
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
