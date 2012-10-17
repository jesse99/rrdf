use functions_on_strings::*;
use test_helpers::*;

#[test]
fn substr2()
{
	let actual = substr2_fn(&StringValue(~"hello", ~""), &IntValue(1i64));
	assert check_operands(&actual, &StringValue(~"hello", ~""));
	
	let actual = substr2_fn(&StringValue(~"hello", ~""), &IntValue(2i64));
	assert check_operands(&actual, &StringValue(~"ello", ~""));
	
	let actual = substr2_fn(&StringValue(~"hello", ~"de"), &IntValue(2i64));
	assert check_operands(&actual, &StringValue(~"ello", ~"de"));
	
	let actual = substr2_fn(&StringValue(~"hello", ~""), &IntValue(4i64));
	assert check_operands(&actual, &StringValue(~"lo", ~""));
	
	let actual = substr2_fn(&StringValue(~"hello", ~""), &IntValue(6i64));
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = substr2_fn(&StringValue(~"hello", ~""), &IntValue(-7i64));
	assert check_operands(&actual, &ErrorValue(~"SUBSTR: startingLoc is -7."));
}

#[test]
fn substr3()
{
	let actual = substr3_fn(&StringValue(~"hello", ~""), &IntValue(1i64), &IntValue(0i64));
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = substr3_fn(&StringValue(~"hello", ~""), &IntValue(2i64), &IntValue(3i64));
	assert check_operands(&actual, &StringValue(~"ell", ~""));
	
	let actual = substr3_fn(&StringValue(~"hello", ~"de"), &IntValue(2i64), &IntValue(3i64));
	assert check_operands(&actual, &StringValue(~"ell", ~"de"));
	
	let actual = substr3_fn(&StringValue(~"hello", ~""), &IntValue(8i64), &IntValue(1i64));
	assert check_operands(&actual, &ErrorValue(~"SUBSTR: startingLoc of 8 and length 1 is past the end of the string."));
	
	let actual = substr3_fn(&StringValue(~"hello", ~""), &IntValue(2i64), &IntValue(100i64));
	assert check_operands(&actual, &ErrorValue(~"SUBSTR: startingLoc of 2 and length 100 is past the end of the string."));
}

#[test]
fn str_before()
{
	let actual = strbefore_fn(&StringValue(~"hello", ~""), &StringValue(~"", ~""));
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = strbefore_fn(&StringValue(~"hello", ~"de"), &StringValue(~"ll", ~""));
	assert check_operands(&actual, &StringValue(~"he", ~"de"));
	
	let actual = strbefore_fn(&StringValue(~"hello", ~""), &StringValue(~"x", ~""));
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = strbefore_fn(&StringValue(~"hello", ~""), &StringValue(~"ll", ~"de"));
	assert check_operands(&actual, &ErrorValue(~"STRBEFORE: '' and 'de' are incompatible languages."));
	
	let actual = strbefore_fn(&StringValue(~"hello", ~"en"), &StringValue(~"ll", ~"de"));
	assert check_operands(&actual, &ErrorValue(~"STRBEFORE: 'en' and 'de' are incompatible languages."));
	
	let actual = strbefore_fn(&StringValue(~"hello", ~"en"), &IntValue(1i64));
	assert check_operands(&actual, &ErrorValue(~"STRBEFORE: expected string for arg2 but found IntValue(1)."));
}

#[test]
fn str_after()
{
	let actual = strafter_fn(&StringValue(~"hello", ~""), &StringValue(~"", ~""));
	assert check_operands(&actual, &StringValue(~"hello", ~""));
	
	let actual = strafter_fn(&StringValue(~"hello", ~"de"), &StringValue(~"ll", ~""));
	assert check_operands(&actual, &StringValue(~"o", ~"de"));
	
	let actual = strafter_fn(&StringValue(~"hello", ~""), &StringValue(~"x", ~""));
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = strafter_fn(&StringValue(~"hello", ~""), &StringValue(~"ll", ~"de"));
	assert check_operands(&actual, &ErrorValue(~"STRAFTER: '' and 'de' are incompatible languages."));
	
	let actual = strafter_fn(&StringValue(~"hello", ~"en"), &StringValue(~"ll", ~"de"));
	assert check_operands(&actual, &ErrorValue(~"STRAFTER: 'en' and 'de' are incompatible languages."));
	
	let actual = strafter_fn(&StringValue(~"hello", ~"en"), &IntValue(1i64));
	assert check_operands(&actual, &ErrorValue(~"STRAFTER: expected string for arg2 but found IntValue(1)."));
}

#[test]
fn encode_for_uri()
{
	let actual = encode_for_uri_fn(&StringValue(~"hello [world]", ~""));
	assert check_operands(&actual, &StringValue(~"hello%20%5Bworld%5D", ~""));
	
	let actual = encode_for_uri_fn(&StringValue(~"Ab0-_.~", ~""));
	assert check_operands(&actual, &StringValue(~"Ab0-_.~", ~""));
}

#[test]
fn concat()
{
	let actual = concat_fn(~[]);
	assert check_operands(&actual, &StringValue(~"", ~""));
	
	let actual = concat_fn(~[StringValue(~"hello ", ~""), StringValue(~"world", ~""), StringValue(~"!", ~"")]);
	assert check_operands(&actual, &StringValue(~"hello world!", ~""));
	
	let actual = concat_fn(~[StringValue(~"hello ", ~"en"), StringValue(~"world", ~"en"), StringValue(~"!", ~"en")]);
	assert check_operands(&actual, &StringValue(~"hello world!", ~"en"));
	
	let actual = concat_fn(~[StringValue(~"hello ", ~"en"), StringValue(~"world", ~"de"), StringValue(~"!", ~"en")]);
	assert check_operands(&actual, &StringValue(~"hello world!", ~""));
	
	let actual = concat_fn(~[StringValue(~"hello", ~"en"), IntValue(1i64), StringValue(~"!", ~"en")]);
	assert check_operands(&actual, &ErrorValue(~"CONCAT: expected string for argument 1 but found IntValue(1)."));
}
