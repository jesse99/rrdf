// Used to test the individual query functions. These aren't necessarily intended to be
// exhaustive, they're just a handy way to unit test components. The exhaustive tests
// are elsewhere, e.g. the test_sparql modules.
use io::WriterUtil;
use query::*;
use test_data::*;
use test_helpers::*;

pub fn check_str_array(actual: &[~str], expected: &[~str]) -> bool
{
	if actual != expected
	{
		io::stderr().write_line("Found:");
		io::stderr().write_line(fmt!("   %?", actual));
		io::stderr().write_line("but expected:");
		io::stderr().write_line(fmt!("   %?", expected));
		return false;
	}
	return true;
}

#[test]
fn test_get_bindings()
{
	let bindings = get_bindings(~[], Group(~[]));
	assert check_str_array(bindings, ~[]);
	
	let bindings = get_bindings(~[~"c", ~"d"], Group(~[]));
	assert check_str_array(bindings, ~[~"c", ~"d"]);
	
	let triple1 = TriplePattern {subject: Variable(~"a"), predicate: Constant(@BoolValue(true)), object: Variable(~"b")};
	let bindings = get_bindings(~[], Basic(copy triple1));
	assert check_str_array(bindings, ~[~"a", ~"b"]);
	
	let bindings = get_bindings(~[~"c", ~"d"], Basic(copy triple1));
	assert check_str_array(bindings, ~[~"c", ~"d", ~"a", ~"b"]);
	
	let triple2 = TriplePattern {subject: Variable(~"a"), predicate: Constant(@BoolValue(true)), object: Variable(~"x")};
	let bindings = get_bindings(~[~"c", ~"d"], Group(~[@Basic(triple1), @Basic(triple2)]));
	assert check_str_array(bindings, ~[~"c", ~"d", ~"a", ~"b", ~"x"]);
}

#[test]
fn test_join()
{
	let store = got_cast3();
	
	// empty x empty
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected1);
	
	// full x empty
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let expected2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected2);
	
	// full x none
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Petyr Baelish", ~""), @UnboundValue, @StringValue(~"Lord", ~"")],
		]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let expected2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected2);
	
	// empty x full
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Jon Snow", ~""), @UnboundValue, @StringValue(~"", ~"")],
			~[@StringValue(~"Eddark Stark", ~""), @UnboundValue, @StringValue(~"Warden of the North", ~"")],
			~[@StringValue(~"Petyr Baelish", ~""), @UnboundValue, @StringValue(~"Lord", ~"")],
		]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	let expected2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected2);
	
	// full x some
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @UnboundValue, @StringValue(~"Warden of the North", ~"")],
		]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @StringValue(~"Warden of the North", ~"")],
		]};
	let expected2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @StringValue(~"Warden of the North", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~""), @UnboundValue],
		]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected2);
	
	// full x all
	let group1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @UnboundValue],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @UnboundValue],
		]};
	let group2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Jon Snow", ~""), @UnboundValue, @StringValue(~"", ~"")],
			~[@StringValue(~"Eddark Stark", ~""), @UnboundValue, @StringValue(~"Warden of the North", ~"")],
			~[@StringValue(~"Petyr Baelish", ~""), @UnboundValue, @StringValue(~"Lord", ~"")],
		]};
	let expected1 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @StringValue(~"Warden of the North", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @StringValue(~"", ~"")],
		]};
	let expected2 = Solution {namespaces: copy store.namespaces, bindings: ~[~"name", ~"nickname", ~"honorific"], num_selected: 3, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~""), @StringValue(~"Warden of the North", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~""), @StringValue(~"", ~"")],
		]};
	assert check_solution(&join_solutions(&store, &group1, &group2, false), &expected1);
	assert check_solution(&join_solutions(&store, &group1, &group2, true), &expected2);
}

#[test]
fn test_basic()
{
	// Variable Variable Variable
	let store = got_cast1();
	let pattern = TriplePattern {subject: Variable(~"subject"), predicate: Variable(~"predicate"), object: Variable(~"value")};
	let bindings = ~[~"value", ~"subject", ~"predicate"];
	let actual = eval_basic(&store, copy bindings, 1, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 1, rows: 
		~[
			~[@StringValue(~"Eddard Stark", ~""), @IriValue(~"http://awoiaf.westeros.org/index.php/Eddard_Stark"), @IriValue(~"http://www.w3.org/2006/vcard/ns#fn")],
			~[@StringValue(~"Ned", ~""), @IriValue(~"http://awoiaf.westeros.org/index.php/Eddard_Stark"), @IriValue(~"http://www.w3.org/2006/vcard/ns#nickname")],
		]};
	assert check_solution(&actual, &expected);
	
	// Variable Variable Constant
	let store = animals();
	let pattern = TriplePattern {subject: Variable(~"subject"), predicate: Variable(~"predicate"), object: Constant(@StringValue(~"mammalia", ~""))};
	let bindings = ~[~"subject", ~"predicate"];
	let actual = eval_basic(&store, copy bindings, 2, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 2, rows: 
		~[
			~[@IriValue(~"http://en.wikipedia.org/wiki/aardvark"), @IriValue(~"http://en.wikipedia.org/wiki/class")],
			~[@IriValue(~"http://en.wikipedia.org/wiki/giraffe"), @IriValue(~"http://en.wikipedia.org/wiki/class")],
			~[@IriValue(~"http://en.wikipedia.org/wiki/grizzly"), @IriValue(~"http://en.wikipedia.org/wiki/class")],
		]};
	assert check_solution(&actual, &expected);
	
	// Variable Constant Variable
	let pattern = TriplePattern {subject: Variable(~"subject"), predicate: Constant(@IriValue(~"http://en.wikipedia.org/wiki/class")), object: Variable(~"value")};
	let bindings = ~[~"value", ~"subject"];
	let actual = eval_basic(&store, copy bindings, 1, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 1, rows: 
		~[
			~[@StringValue(~"actinopterygii", ~""), @IriValue(~"http://en.wikipedia.org/wiki/salmon")],
			~[@StringValue(~"arachnida", ~""), @IriValue(~"http://en.wikipedia.org/wiki/black_widow")],
			~[@StringValue(~"insecta", ~""), @IriValue(~"http://en.wikipedia.org/wiki/firefly")],
			~[@StringValue(~"mammalia", ~""), @IriValue(~"http://en.wikipedia.org/wiki/aardvark")],
			~[@StringValue(~"mammalia", ~""), @IriValue(~"http://en.wikipedia.org/wiki/giraffe")],
			~[@StringValue(~"mammalia", ~""), @IriValue(~"http://en.wikipedia.org/wiki/grizzly")],
		]};
	assert check_solution(&actual, &expected);
	
	// Constant Variable Variable
	let pattern = TriplePattern {subject: Constant(@IriValue(~"http://en.wikipedia.org/wiki/black_widow")), predicate: Variable(~"predicate"), object: Variable(~"value")};
	let bindings = ~[~"value", ~"predicate"];
	let actual = eval_basic(&store, copy bindings, 1, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 1, rows: 
		~[
			~[@StringValue(~"arachnida", ~""), @IriValue(~"http://en.wikipedia.org/wiki/class")],
			~[@StringValue(~"arthropoda", ~""), @IriValue(~"http://en.wikipedia.org/wiki/phylum")],
			~[@StringValue(~"theridiidae", ~""), @IriValue(~"http://en.wikipedia.org/wiki/family")],
		]};
	assert check_solution(&actual, &expected);
	
	// Constant Variable Constant
	let pattern = TriplePattern {subject: Constant(@IriValue(~"http://en.wikipedia.org/wiki/black_widow")), predicate: Variable(~"predicate"), object: Constant(@StringValue(~"arthropoda", ~""))};
	let bindings = ~[~"predicate"];
	let actual = eval_basic(&store, copy bindings, 1, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 1, rows: 
		~[
			~[@IriValue(~"http://en.wikipedia.org/wiki/phylum")],
		]};
	assert check_solution(&actual, &expected);
	
	// Constant Constant Constant
	let pattern = TriplePattern {subject: Constant(@IriValue(~"http://en.wikipedia.org/wiki/black_widow")), predicate: Constant(@IriValue(~"http://en.wikipedia.org/wiki/phylum")), object: Constant(@StringValue(~"arthropoda", ~""))};
	let bindings = ~[];
	let actual = eval_basic(&store, copy bindings, 0, &pattern);
	let expected = Solution {namespaces: copy store.namespaces, bindings: bindings, num_selected: 0, rows: 
		~[
			~[],
		]};
	assert check_solution(&actual, &expected);
}

#[test]
fn test_bind()
{
	let context = QueryContext {namespaces: ~[], extensions: HashMap(), algebra: Group(~[]), order_by: ~[], distinct: false, limit: option::None, rng: rand::Rng(), timestamp: time::now()};
	let bindings = ~[~"subject", ~"predicate", ~"value", ~"x"];
	let solution = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject0"), @IriValue(~"predicate0"), @StringValue(~"value0", ~""), @UnboundValue],
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~""), @UnboundValue],
			~[@IriValue(~"subject2"), @IriValue(~"predicate2"), @StringValue(~"value2", ~""), @UnboundValue],
		]};
	
	// constant
	let mut actual = copy solution;
	let expr = ConstantExpr(IntValue(42));
	let result = bind_solution(&context, &mut actual, &expr, ~"x");
	assert result.is_none();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject0"), @IriValue(~"predicate0"), @StringValue(~"value0", ~""), @IntValue(42)],
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~""), @IntValue(42)],
			~[@IriValue(~"subject2"), @IriValue(~"predicate2"), @StringValue(~"value2", ~""), @IntValue(42)],
		]};
	assert check_solution(&actual, &expected);
	
	// variable
	let mut actual = copy solution;
	let expr = VariableExpr(~"predicate");
	let result = bind_solution(&context, &mut actual, &expr, ~"x");
	assert result.is_none();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject0"), @IriValue(~"predicate0"), @StringValue(~"value0", ~""), @IriValue(~"predicate0")],
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~""), @IriValue(~"predicate1")],
			~[@IriValue(~"subject2"), @IriValue(~"predicate2"), @StringValue(~"value2", ~""), @IriValue(~"predicate2")],
		]};
	assert check_solution(&actual, &expected);
	
	// call + variable
	let mut actual = copy solution;
	let expr = CallExpr(~"ucase_fn", ~[@VariableExpr(~"value")]);
	let result = bind_solution(&context, &mut actual, &expr, ~"x");
	assert result.is_none();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject0"), @IriValue(~"predicate0"), @StringValue(~"value0", ~""), @StringValue(~"VALUE0", ~"")],
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~""), @StringValue(~"VALUE1", ~"")],
			~[@IriValue(~"subject2"), @IriValue(~"predicate2"), @StringValue(~"value2", ~""), @StringValue(~"VALUE2", ~"")],
		]};
	assert check_solution(&actual, &expected);
}

#[test]
fn test_filter()
{
	let context = QueryContext {namespaces: ~[], extensions: HashMap(), algebra: Group(~[]), order_by: ~[], distinct: false, limit: option::None, rng: rand::Rng(), timestamp: time::now()};
	let bindings = ~[~"subject", ~"predicate", ~"value"];
	let solution = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject0"), @IriValue(~"predicate0"), @StringValue(~"value0", ~"")],
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~"")],
			~[@IriValue(~"subject2"), @IriValue(~"predicate2"), @StringValue(~"value2", ~"")],
		]};
	
	let mut actual = copy solution;
	let expr = CallExpr(~"strends_fn", ~[@VariableExpr(~"value"), @ConstantExpr(StringValue(~"1", ~""))]);
	let result = filter_solution(&context, &mut actual, &expr);
	assert result.is_none();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 3, rows: 
		~[
			~[@IriValue(~"subject1"), @IriValue(~"predicate1"), @StringValue(~"value1", ~"")],
		]};
	assert check_solution(&actual, &expected);
}

// This is starting to get complex so we'll do the bulk of eval_group and eval_algebra testing in test_sparql.rs
#[test]
fn test_group()
{
	// basic + basic
	let store = got_cast3();
	let p1 = @Basic(TriplePattern {subject: Variable(~"subject"), predicate: Constant(@IriValue(~"http://www.w3.org/2006/vcard/ns#nickname")), object: Variable(~"name")});
	let p2 = @Basic(TriplePattern {subject: Variable(~"subject"), predicate: Constant(@IriValue(~"http://www.w3.org/2006/vcard/ns#honorific-prefix")), object: Variable(~"title")});
	let context = QueryContext {namespaces: ~[], extensions: HashMap(), algebra: Group(~[]), order_by: ~[], distinct: false, limit: option::None, rng: rand::Rng(), timestamp: time::now()};
	let bindings = ~[~"name", ~"title", ~"subject"];
	let actual = eval_group(&store, &context, copy bindings, 2, &[p1, p2]);
	assert actual.is_ok();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 2, rows: 
		~[
			~[@StringValue(~"Ned", ~""), @StringValue(~"Lord", ~""), @IriValue(~"http://awoiaf.westeros.org/index.php/Eddard_Stark")],
		]};
	assert check_solution(result::get_ref(&actual), &expected);
}

#[test]
fn test_order_by()
{
	let context = QueryContext {namespaces: ~[], extensions: HashMap(), algebra: Group(~[]), order_by: ~[], distinct: false, limit: option::None, rng: rand::Rng(), timestamp: time::now()};
	let bindings = ~[~"name", ~"nickname"];
	let mut solution = Solution {namespaces: ~[], bindings: bindings, num_selected: 2, rows: 
		~[
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Tiny", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Dog", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
		]};
	let exprs = ~[VariableExpr(~"name"), VariableExpr(~"nickname")];
	let actual = order_by(&context, &solution, exprs);
	assert actual.is_ok();
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 2, rows: 
		~[
			~[@StringValue(~"Eddark Stark", ~""), @StringValue(~"Ned", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Dog", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Tiny", ~"")],
		]};
	assert check_solution(result::get_ref(&actual), &expected);
}

#[test]
fn test_distinct()
{
	let bindings = ~[~"name", ~"nickname"];
	let solution = Solution {namespaces: ~[], bindings: bindings, num_selected: 2, rows: 
		~[
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Dog", ~"")],
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
		]};
	let expected = Solution {namespaces: ~[], bindings: bindings, num_selected: 2, rows: 
		~[
			~[@StringValue(~"Jon Snow", ~""), @StringValue(~"Lord Snow", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"Dog", ~"")],
			~[@StringValue(~"Sandor Clegane", ~""), @StringValue(~"The Hound", ~"")],
		]};
	let actual = make_distinct(&solution);
	assert check_solution(&actual, &expected);
}
