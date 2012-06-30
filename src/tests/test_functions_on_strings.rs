import functions_on_strings::*;
import test_helpers::*;

#[test]
fn substr2()
{
	let actual = substr2_fn(string_value("hello", ""), int_value(1i64));
	assert check_operands(actual, string_value("hello", ""));
	
	let actual = substr2_fn(string_value("hello", ""), int_value(2i64));
	assert check_operands(actual, string_value("ello", ""));
	
	let actual = substr2_fn(string_value("hello", "de"), int_value(2i64));
	assert check_operands(actual, string_value("ello", "de"));
	
	let actual = substr2_fn(string_value("hello", ""), int_value(4i64));
	assert check_operands(actual, string_value("lo", ""));
	
	let actual = substr2_fn(string_value("hello", ""), int_value(6i64));
	assert check_operands(actual, string_value("", ""));
	
	let actual = substr2_fn(string_value("hello", ""), int_value(-7i64));
	assert check_operands(actual, error_value("SUBSTR: startingLoc is -7."));
}

#[test]
fn substr3()
{
	let actual = substr3_fn(string_value("hello", ""), int_value(1i64), int_value(0i64));
	assert check_operands(actual, string_value("", ""));
	
	let actual = substr3_fn(string_value("hello", ""), int_value(2i64), int_value(3i64));
	assert check_operands(actual, string_value("ell", ""));
	
	let actual = substr3_fn(string_value("hello", "de"), int_value(2i64), int_value(3i64));
	assert check_operands(actual, string_value("ell", "de"));
	
	let actual = substr3_fn(string_value("hello", ""), int_value(8i64), int_value(1i64));
	assert check_operands(actual, error_value("SUBSTR: startingLoc of 8 and length 1 is past the end of the string."));
	
	let actual = substr3_fn(string_value("hello", ""), int_value(2i64), int_value(100i64));
	assert check_operands(actual, error_value("SUBSTR: startingLoc of 2 and length 100 is past the end of the string."));
}

#[test]
fn str_before()
{
	let actual = strbefore_fn(string_value("hello", ""), string_value("", ""));
	assert check_operands(actual, string_value("", ""));
	
	let actual = strbefore_fn(string_value("hello", "de"), string_value("ll", ""));
	assert check_operands(actual, string_value("he", "de"));
	
	let actual = strbefore_fn(string_value("hello", ""), string_value("x", ""));
	assert check_operands(actual, string_value("", ""));
	
	let actual = strbefore_fn(string_value("hello", ""), string_value("ll", "de"));
	assert check_operands(actual, error_value("STRBEFORE: '' and 'de' are incompatible languages."));
	
	let actual = strbefore_fn(string_value("hello", "en"), string_value("ll", "de"));
	assert check_operands(actual, error_value("STRBEFORE: 'en' and 'de' are incompatible languages."));
	
	let actual = strbefore_fn(string_value("hello", "en"), int_value(1i64));
	assert check_operands(actual, error_value("STRBEFORE: expected string for arg2 but found int_value(1)."));
}

#[test]
fn str_after()
{
	let actual = strafter_fn(string_value("hello", ""), string_value("", ""));
	assert check_operands(actual, string_value("hello", ""));
	
	let actual = strafter_fn(string_value("hello", "de"), string_value("ll", ""));
	assert check_operands(actual, string_value("o", "de"));
	
	let actual = strafter_fn(string_value("hello", ""), string_value("x", ""));
	assert check_operands(actual, string_value("", ""));
	
	let actual = strafter_fn(string_value("hello", ""), string_value("ll", "de"));
	assert check_operands(actual, error_value("STRAFTER: '' and 'de' are incompatible languages."));
	
	let actual = strafter_fn(string_value("hello", "en"), string_value("ll", "de"));
	assert check_operands(actual, error_value("STRAFTER: 'en' and 'de' are incompatible languages."));
	
	let actual = strafter_fn(string_value("hello", "en"), int_value(1i64));
	assert check_operands(actual, error_value("STRAFTER: expected string for arg2 but found int_value(1)."));
}

#[test]
fn encode_for_uri()
{
	let actual = encode_for_uri_fn(string_value("hello [world]", ""));
	assert check_operands(actual, string_value("hello%20%5Bworld%5D", ""));
	
	let actual = encode_for_uri_fn(string_value("Ab0-_.~", ""));
	assert check_operands(actual, string_value("Ab0-_.~", ""));
}

#[test]
fn concat()
{
	let actual = concat_fn([]);
	assert check_operands(actual, string_value("", ""));
	
	let actual = concat_fn([string_value("hello ", ""), string_value("world", ""), string_value("!", "")]);
	assert check_operands(actual, string_value("hello world!", ""));
	
	let actual = concat_fn([string_value("hello ", "en"), string_value("world", "en"), string_value("!", "en")]);
	assert check_operands(actual, string_value("hello world!", "en"));
	
	let actual = concat_fn([string_value("hello ", "en"), string_value("world", "de"), string_value("!", "en")]);
	assert check_operands(actual, string_value("hello world!", ""));
	
	let actual = concat_fn([string_value("hello", "en"), int_value(1i64), string_value("!", "en")]);
	assert check_operands(actual, error_value("CONCAT: expected string for argument 1 but found int_value(1)."));
}
