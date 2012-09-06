//! SPARQL functions. Clients will not ordinarily use this.
use object::*;

fn isiri_fn(operand: Object) -> Object
{
	match operand
	{
		IriValue(_name) =>
		{
			BoolValue(true)
		}
		_ =>
		{
			BoolValue(false)
		}
	}
}

fn isblank_fn(operand: Object) -> Object
{
	match operand
	{
		BlankValue(_name) =>
		{
			BoolValue(true)
		}
		_ =>
		{
			BoolValue(false)
		}
	}
}

fn isliteral_fn(operand: Object) -> Object
{
	match operand
	{
		BoolValue(*) |  IntValue(*) | FloatValue(*) | DateTimeValue(*) |StringValue(*) | TypedValue(*) =>
		{
			BoolValue(true)
		}
		_ =>
		{
			BoolValue(false)
		}
	}
}

fn isnumeric_fn(operand: Object) -> Object
{
	match operand
	{
		IntValue(*) | FloatValue(*) =>
		{
			BoolValue(true)
		}
		_ =>
		{
			BoolValue(false)
		}
	}
}

fn str_fn(operand: Object) -> Object
{
	StringValue(operand.to_str(), ~"")
}

fn lang_fn(operand: Object) -> Object
{
	match operand
	{
		StringValue(_value, lang) =>
		{
			StringValue(lang, ~"")
		}
		_ =>
		{
			StringValue(~"", ~"")
		}
	}
}

fn datatype_fn(operand: Object) -> Object
{
	match operand
	{
		BoolValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#boolean", ~"")
		}
		IntValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#integer", ~"")
		}
		FloatValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#double", ~"")
		}
		DateTimeValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#dateTime", ~"")
		}
		StringValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#string", ~"")
		}
		TypedValue(_value, kind) =>
		{
			StringValue(kind, ~"")
		}
		IriValue(*) =>
		{
			StringValue(~"http://www.w3.org/2001/XMLSchema#anyURI", ~"")
		}
		_ =>
		{
			ErrorValue(fmt!("DATATYPE: can't get a type for %?", operand))
		}
	}
}

// TODO: add iri_fn
// TODO: add bnode_fn

fn strdt_fn(lexical: Object, kind: Object) -> Object
{
	match lexical
	{
		BoolValue(*) | IntValue(*) | FloatValue(*) | DateTimeValue(*) | StringValue(*) =>
		{
			match kind
			{
				IriValue(value) =>
				{
					TypedValue(lexical.to_str(), value)
				}
				_ =>
				{
					ErrorValue(fmt!("STRDT: expected an IRI for the second argument found %?", kind))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("STRDT: expected a simple literal for the first argument but found %?", lexical))
		}
	}
}

fn strlang_fn(lexical: Object, tag: Object) -> Object
{
	match lexical
	{
		BoolValue(*) | IntValue(*) | FloatValue(*) | DateTimeValue(*) | StringValue(*) =>
		{
			match tag
			{
				BoolValue(*) | IntValue(*) | FloatValue(*) | DateTimeValue(*) | StringValue(*) =>
				{
					StringValue(lexical.to_str(), tag.to_str())
				}
				_ =>
				{
					ErrorValue(fmt!("STRLANG: expected a simple literal for the second argument found %?", tag))
				}
			}
		}
		_ =>
		{
			ErrorValue(fmt!("STRLANG: expected a simple literal for the first argument but found %?", lexical))
		}
	}
}
