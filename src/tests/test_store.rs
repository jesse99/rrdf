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

// TODO:
// should be able to use each by importing iter-traits
// check that add_triples converts references
// probably should check blank nodes (use got_cast3)
// get_blank_name
// query
// remove
// clean
// add_store
