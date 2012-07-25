#[doc = "SPARQL functions. Clients will not ordinarily use this."];

export strlen_fn, substr2_fn, substr3_fn, ucase_fn, lcase_fn, strstarts_fn, strends_fn, contains_fn, strbefore_fn, 
strafter_fn, encode_for_uri_fn, concat_fn, langmatches_fn;

fn str_str_helper(fname: ~str, arg1: object, arg2: object, callback: fn@ (~str, ~str, ~str, ~str) -> object) -> object
{
	alt arg1
	{
		string_value(value1, lang1)
		{
			alt arg2
			{
				string_value(value2, lang2)
				{
					if str::to_lower(lang1) == str::to_lower(lang2) || str::is_empty(lang2)
					{
						callback(value1, value2, lang1, lang2)
					}
					else
					{
						error_value(#fmt["%s: '%s' and '%s' are incompatible languages.", fname, lang1, lang2])
					}
				}
				_
				{
					error_value(#fmt["%s: expected string for arg2 but found %?.", fname, arg2])
				}
			}
		}
		_
		{
			error_value(#fmt["%s: expected string for arg1 but found %?.", fname, arg1])
		}
	}
}

fn strlen_fn(operand: object) -> object
{
	alt operand
	{
		string_value(value, _lang)
		{
			int_value(str::len(value) as i64)
		}
		_
		{
			error_value(#fmt["STRLEN: expected string but found %?.", operand])
		}
	}
}

fn substr2_fn(value: object, loc: object) -> object
{
	alt value
	{
		string_value(source, lang)
		{
			alt loc
			{
				int_value(startingLoc)
				{
					let begin = (startingLoc - 1i64) as uint;		// for some stupid reason the indexes are 1-based
					let end = str::len(source);
					if startingLoc >= 1i64 && begin <= end
					{
						string_value(str::slice(source, begin, end), lang)
					}
					else if startingLoc == 0i64
					{
						error_value(#fmt["SUBSTR: startingLoc should be 1 or larger not %?.", startingLoc])
					}
					else if startingLoc < 0i64
					{
						error_value(#fmt["SUBSTR: startingLoc is %?.", startingLoc])
					}
					else
					{
						error_value(#fmt["SUBSTR: startingLoc of %? is past the end of the string.", startingLoc])
					}
				}
				_
				{
					error_value(#fmt["SUBSTR: expected int for startingLoc but found %?.", loc])
				}
			}
		}
		_
		{
			error_value(#fmt["SUBSTR: expected string for source but found %?.", value])
		}
	}
}

fn substr3_fn(value: object, loc: object, len: object) -> object
{
	alt value
	{
		string_value(source, lang)
		{
			alt loc
			{
				int_value(startingLoc)
				{
					alt len
					{
						int_value(length)
						{
							let begin = (startingLoc - 1i64) as uint;		// for some stupid reason the indexes are 1-based
							let end = begin + length as uint;
							if startingLoc >= 1i64 && end <= str::len(source)
							{
								string_value(str::slice(source, begin, end), lang)
							}
							else if startingLoc == 0i64
							{
								error_value(#fmt["SUBSTR: startingLoc should be 1 or larger not %?.", startingLoc])
							}
							else if startingLoc < 0i64
							{
								error_value(#fmt["SUBSTR: startingLoc is %?.", startingLoc])
							}
							else
							{
								error_value(#fmt["SUBSTR: startingLoc of %? and length %? is past the end of the string.", startingLoc, length])
							}
						}
						_
						{
							error_value(#fmt["SUBSTR: expected int for length but found %?.", len])
						}
					}
				}
				_
				{
					error_value(#fmt["SUBSTR: expected int for startingLoc but found %?.", loc])
				}
			}
		}
		_
		{
			error_value(#fmt["SUBSTR: expected string for source but found %?.", value])
		}
	}
}

fn ucase_fn(operand: object) -> object
{
	alt operand
	{
		string_value(value, lang)
		{
			string_value(str::to_upper(value), lang)
		}
		_
		{
			error_value(#fmt["UCASE: expected string but found %?.", operand])
		}
	}
}

fn lcase_fn(operand: object) -> object
{
	alt operand
	{
		string_value(value, lang)
		{
			string_value(str::to_lower(value), lang)
		}
		_
		{
			error_value(#fmt["LCASE: expected string but found %?.", operand])
		}
	}
}

fn strstarts_fn(arg1: object, arg2: object) -> object
{
	do str_str_helper(~"STRSTARTS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		bool_value(str::starts_with(value1, value2))
	}
}

fn strends_fn(arg1: object, arg2: object) -> object
{
	do str_str_helper(~"STRENDS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		bool_value(str::ends_with(value1, value2))
	}
}

fn contains_fn(arg1: object, arg2: object) -> object
{
	do str_str_helper(~"CONTAINS", arg1, arg2)
	|value1, value2, _lang1, _lang2|
	{
		bool_value(str::contains(value1, value2))
	}
}

fn strbefore_fn(arg1: object, arg2: object) -> object
{
	do str_str_helper(~"STRBEFORE", arg1, arg2)
	|value1, value2, lang1, _lang2|
	{
		alt str::find_str(value1, value2)
		{
			option::some(i)
			{
				string_value(str::slice(value1, 0u, i), lang1)
			}
			option::none
			{
				string_value(~"", ~"")		// this changed post 1.1
			}
		}
	}
}

fn strafter_fn(arg1: object, arg2: object) -> object
{
	do str_str_helper(~"STRAFTER", arg1, arg2)
	|value1, value2, lang1, _lang2|
	{
		alt str::find_str(value1, value2)
		{
			option::some(i)
			{
				string_value(str::slice(value1, i + str::len(value2), str::len(value1)), lang1)
			}
			option::none
			{
				string_value(~"", ~"")	// this changed post 1.1
			}
		}
	}
}

fn is_unreserved(ch: char) -> bool
{
	if ch >= 'a' && ch <= 'z'
	{
		ret true;
	}
	else if ch >= 'A' && ch <= 'Z'
	{
		ret true;
	}
	else if ch >= '0' && ch <= '9'
	{
		ret true;
	}
	else if ch == '-' || ch == '_' || ch == '.' || ch == '~'
	{
		ret true;
	}
	else
	{
		ret false;
	}
}

fn encode_for_uri_fn(operand: object) -> object
{
	alt operand
	{
		string_value(value, lang)
		{
			let mut result = ~"";
			str::reserve(result, str::len(value));
			
			for str::each_char(value)
			|ch|
			{
				if is_unreserved(ch)
				{
					str::push_char(result, ch);
				}
				else
				{
					result += #fmt["%%%0X", ch as uint];
				}
			}
			
			string_value(result, lang)
		}
		_
		{
			error_value(#fmt["ENCODE_FOR_URI: expected string but found %?.", operand])
		}
	}
}

fn concat_fn(operand: ~[object]) -> object
{
	let mut result = ~"";
	let mut languages = ~[];
	
	for vec::eachi(operand)
	|i, part|
	{
		alt part
		{
			string_value(value, lang)
			{
				result += value;
				if !vec::contains(languages, lang)
				{
					vec::push(languages, lang);
				}
			}
			_
			{
				ret error_value(#fmt["CONCAT: expected string for argument %? but found %?.", i, part]);
			}
		}
	}
	
	if vec::len(languages) == 1u
	{
		string_value(result, languages[0])
	}
	else
	{
		string_value(result, ~"")
	}
}

fn langmatches_fn(arg1: object, arg2: object) -> object
{
	alt arg1
	{
		string_value(_value1, lang1)
		{
			alt arg2
			{
				string_value(_value2, lang2)
				{
					bool_value(str::to_lower(lang1) == str::to_lower(lang2))
				}
				_
				{
					error_value(#fmt["LANGMATCHES: expected string for arg2 but found %?.", arg2])
				}
			}
		}
		_
		{
			error_value(#fmt["LANGMATCHES: expected string for arg1 but found %?.", arg1])
		}
	}
}

// TODO: add regex and replace
