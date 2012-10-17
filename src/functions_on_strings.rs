//! SPARQL functions. Clients will not ordinarily use this.

pub fn str_str_helper(fname: ~str, arg1: &Object, arg2: &Object, callback: fn@ (&str, &str, &str, &str) -> Object) -> Object
{
	match *arg1
	{
		StringValue(ref value1, ref lang1) =>
		{
			match *arg2
			{
				StringValue(ref value2, ref lang2) =>
				{
					if str::to_lower(*lang1) == str::to_lower(*lang2) || str::is_empty(*lang2)
					{
						callback(*value1, *value2, *lang1, *lang2)
					}
					else
					{
						ErrorValue(fmt!("%s: '%s' and '%s' are incompatible languages.", fname, *lang1, *lang2))
					}
				}
				_ =>
				{
					ErrorValue(fmt!("%s: expected string for arg2 but found %?.", fname, *arg2))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("%s: expected string for arg1 but found %?.", fname, *arg1))
		}
	}
}

pub fn strlen_fn(operand: &Object) -> Object
{
	match *operand
	{
		StringValue(ref value, ref _lang) =>
		{
			IntValue(str::len(*value) as i64)
		}
		_ =>
		{
			ErrorValue(fmt!("STRLEN: expected string but found %?.", *operand))
		}
	}
}

pub fn substr2_fn(value: &Object, loc: &Object) -> Object
{
	match *value
	{
		StringValue(ref source, copy lang) =>
		{
			match *loc
			{
				IntValue(startingLoc) =>
				{
					let begin = (startingLoc - 1i64) as uint;		// for some stupid reason the indexes are 1-based
					let end = str::len(*source);
					if startingLoc >= 1i64 && begin <= end
					{
						StringValue(str::slice(*source, begin, end), lang)
					}
					else if startingLoc == 0i64
					{
						ErrorValue(fmt!("SUBSTR: startingLoc should be 1 or larger not %?.", startingLoc))
					}
					else if startingLoc < 0i64
					{
						ErrorValue(fmt!("SUBSTR: startingLoc is %?.", startingLoc))
					}
					else
					{
						ErrorValue(fmt!("SUBSTR: startingLoc of %? is past the end of the string.", startingLoc))
					}
				}
				_ =>
				{
					ErrorValue(fmt!("SUBSTR: expected int for startingLoc but found %?.",* loc))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("SUBSTR: expected string for source but found %?.", *value))
		}
	}
}

pub fn substr3_fn(value: &Object, loc: &Object, len: &Object) -> Object
{
	match *value
	{
		StringValue(ref source, copy lang) =>
		{
			match *loc
			{
				IntValue(startingLoc) =>
				{
					match *len
					{
						IntValue(length) =>
						{
							let begin = (startingLoc - 1i64) as uint;		// for some stupid reason the indexes are 1-based
							let end = begin + length as uint;
							if startingLoc >= 1i64 && end <= str::len(*source)
							{
								StringValue(str::slice(*source, begin, end), lang)
							}
							else if startingLoc == 0i64
							{
								ErrorValue(fmt!("SUBSTR: startingLoc should be 1 or larger not %?.", startingLoc))
							}
							else if startingLoc < 0i64
							{
								ErrorValue(fmt!("SUBSTR: startingLoc is %?.", startingLoc))
							}
							else
							{
								ErrorValue(fmt!("SUBSTR: startingLoc of %? and length %? is past the end of the string.", startingLoc, length))
							}
						}
						_ =>
						{
							ErrorValue(fmt!("SUBSTR: expected int for length but found %?.", *len))
						}
					}
				}
				_ =>
				{
					ErrorValue(fmt!("SUBSTR: expected int for startingLoc but found %?.", *loc))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("SUBSTR: expected string for source but found %?.", *value))
		}
	}
}

pub fn ucase_fn(operand: &Object) -> Object
{
	match *operand
	{
		StringValue(ref value, copy lang) =>
		{
			StringValue(str::to_upper(*value), lang)
		}
		_ =>
		{
			ErrorValue(fmt!("UCASE: expected string but found %?.", *operand))
		}
	}
}

pub fn lcase_fn(operand: &Object) -> Object
{
	match *operand
	{
		StringValue(ref value, copy lang) =>
		{
			StringValue(str::to_lower(*value), lang)
		}
		_ =>
		{
			ErrorValue(fmt!("LCASE: expected string but found %?.", *operand))
		}
	}
}

pub fn strstarts_fn(arg1: &Object, arg2: &Object) -> Object
{
	do str_str_helper(~"STRSTARTS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		BoolValue(str::starts_with(value1, value2))
	}
}

pub fn strends_fn(arg1: &Object, arg2: &Object) -> Object
{
	do str_str_helper(~"STRENDS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		BoolValue(str::ends_with(value1, value2))
	}
}

pub fn contains_fn(arg1: &Object, arg2: &Object) -> Object
{
	do str_str_helper(~"CONTAINS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		BoolValue(str::contains(value1, value2))
	}
}

pub fn strbefore_fn(arg1: &Object, arg2: &Object) -> Object
{
	do str_str_helper(~"STRBEFORE", arg1, arg2)
	|value1, value2, lang1, _lang2|
	{
		match str::find_str(value1, value2)
		{
			option::Some(i) =>
			{
				StringValue(str::slice(value1, 0u, i), lang1.to_unique())
			}
			option::None =>
			{
				StringValue(~"", ~"")		// this changed post 1.1
			}
		}
	}
}

pub fn strafter_fn(arg1: &Object, arg2: &Object) -> Object
{
	do str_str_helper(~"STRAFTER", arg1, arg2)
	|value1, value2, lang1, _lang2|
	{
		match str::find_str(value1, value2)
		{
			option::Some(i) =>
			{
				StringValue(str::slice(value1, i + str::len(value2), str::len(value1)), lang1.to_unique())
			}
			option::None =>
			{
				StringValue(~"", ~"")	// this changed post 1.1
			}
		}
	}
}

fn is_unreserved(ch: char) -> bool
{
	if ch >= 'a' && ch <= 'z'
	{
		return true;
	}
	else if ch >= 'A' && ch <= 'Z'
	{
		return true;
	}
	else if ch >= '0' && ch <= '9'
	{
		return true;
	}
	else if ch == '-' || ch == '_' || ch == '.' || ch == '~'
	{
		return true;
	}
	else
	{
		return false;
	}
}

pub fn encode_for_uri_fn(operand: &Object) -> Object
{
	match *operand
	{
		StringValue(ref value, copy lang) =>
		{
			let mut result = ~"";
			str::reserve(&mut result, str::len(*value));
			
			for str::each_char(*value)
			|ch|
			{
				if is_unreserved(ch)
				{
					str::push_char(&mut result, ch);
				}
				else
				{
					result += fmt!("%%%0X", ch as uint);
				}
			}
			
			StringValue(result, lang)
		}
		_ =>
		{
			ErrorValue(fmt!("ENCODE_FOR_URI: expected string but found %?.", *operand))
		}
	}
}

pub fn concat_fn(operand: ~[Object]) -> Object
{
	let mut result = ~"";
	let mut languages = ~[];
	
	for vec::eachi(operand)
	|i, part|
	{
		match *part
		{
			StringValue(ref value, copy lang) =>
			{
				result += *value;
				if !vec::contains(languages, &lang)
				{
					vec::push(&mut languages, lang);
				}
			}
			_ =>
			{
				return ErrorValue(fmt!("CONCAT: expected string for argument %? but found %?.", i, *part));
			}
		}
	}
	
	if vec::len(languages) == 1u
	{
		StringValue(result, copy languages[0])
	}
	else
	{
		StringValue(result, ~"")
	}
}

pub fn langmatches_fn(arg1: &Object, arg2: &Object) -> Object
{
	match *arg1
	{
		StringValue(ref _value1, ref lang1) =>
		{
			match *arg2
			{
				StringValue(ref _value2, ref lang2) =>
				{
					BoolValue(str::to_lower(*lang1) == str::to_lower(*lang2))
				}
				_ =>
				{
					ErrorValue(fmt!("LANGMATCHES: expected string for arg2 but found %?.", *arg2))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("LANGMATCHES: expected string for arg1 but found %?.", *arg1))
		}
	}
}

// TODO: add regex and replace
