//! SPARQL functions. Clients will not ordinarily use this.
use object::*;

fn isiri_fn(operand: object) -> object
{
	match operand
	{
		iri_value(_name) =>
		{
			bool_value(true)
		}
		_ =>
		{
			bool_value(false)
		}
	}
}

fn isblank_fn(operand: object) -> object
{
	match operand
	{
		blank_value(_name) =>
		{
			bool_value(true)
		}
		_ =>
		{
			bool_value(false)
		}
	}
}

fn isliteral_fn(operand: object) -> object
{
	match operand
	{
		bool_value(*) |  int_value(*) | float_value(*) | dateTime_value(*) |string_value(*) | typed_value(*) =>
		{
			bool_value(true)
		}
		_ =>
		{
			bool_value(false)
		}
	}
}

fn isnumeric_fn(operand: object) -> object
{
	match operand
	{
		int_value(*) | float_value(*) =>
		{
			bool_value(true)
		}
		_ =>
		{
			bool_value(false)
		}
	}
}

fn str_fn(operand: object) -> object
{
	string_value(operand.to_str(), ~"")
}

fn lang_fn(operand: object) -> object
{
	match operand
	{
		string_value(_value, lang) =>
		{
			string_value(lang, ~"")
		}
		_ =>
		{
			string_value(~"", ~"")
		}
	}
}

fn datatype_fn(operand: object) -> object
{
	match operand
	{
		bool_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#boolean", ~"")
		}
		int_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#integer", ~"")
		}
		float_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#double", ~"")
		}
		dateTime_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#dateTime", ~"")
		}
		string_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#string", ~"")
		}
		typed_value(_value, kind) =>
		{
			string_value(kind, ~"")
		}
		iri_value(*) =>
		{
			string_value(~"http://www.w3.org/2001/XMLSchema#anyURI", ~"")
		}
		_ =>
		{
			error_value(fmt!("DATATYPE: can't get a type for %?", operand))
		}
	}
}

// TODO: add iri_fn
// TODO: add bnode_fn

fn strdt_fn(lexical: object, kind: object) -> object
{
	match lexical
	{
		bool_value(*) | int_value(*) | float_value(*) | dateTime_value(*) | string_value(*) =>
		{
			match kind
			{
				iri_value(value) =>
				{
					typed_value(lexical.to_str(), value)
				}
				_ =>
				{
					error_value(fmt!("STRDT: expected an IRI for the second argument found %?", kind))
				}
			}
		}
		_ =>
		{
			error_value(fmt!("STRDT: expected a simple literal for the first argument but found %?", lexical))
		}
	}
}

fn strlang_fn(lexical: object, tag: object) -> object
{
	match lexical
	{
		bool_value(*) | int_value(*) | float_value(*) | dateTime_value(*) | string_value(*) =>
		{
			match tag
			{
				bool_value(*) | int_value(*) | float_value(*) | dateTime_value(*) | string_value(*) =>
				{
					string_value(lexical.to_str(), tag.to_str())
				}
				_ =>
				{
					error_value(fmt!("STRLANG: expected a simple literal for the second argument found %?", tag))
				}
			}
		}
		_ =>
		{
			error_value(fmt!("STRLANG: expected a simple literal for the first argument but found %?", lexical))
		}
	}
}
