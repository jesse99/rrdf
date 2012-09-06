//! Compiles a SRARQL query into a function that can be applied to a store value.
use std::map::*;
use std::time;
use rparse::rparse::*;
//use expression::*;
use query::*;
use store::*;
use object::*;

export selector, compile;

fn bool_literal(value: @~str) -> pattern
{
	constant(literal_to_object(value, @~"http://www.w3.org/2001/XMLSchema#boolean", @~""))
}

fn int_literal(value: @~str) -> object
{
	literal_to_object(value, @~"http://www.w3.org/2001/XMLSchema#integer", @~"")
}

fn float_literal(value: @~str) -> object
{
	literal_to_object(value, @~"http://www.w3.org/2001/XMLSchema#double", @~"")
}

fn string_literal(value: @~str, lang: @~str) -> pattern
{
	constant(literal_to_object(value, @~"http://www.w3.org/2001/XMLSchema#string", lang))
}

fn typed_literal(value: @~str, kind: @~str) -> pattern
{
	constant(literal_to_object(value, kind, @~""))
}

fn iri_literal(value: @~str) -> pattern
{
	constant(literal_to_object(value, @~"http://www.w3.org/2001/XMLSchema#anyURI", @~""))
}

fn pattern_to_expr(pattern: pattern) -> expr
{
	match pattern
	{
		variable(name) =>
		{
			variable_expr(name)
		}
		constant(value) =>
		{
			constant_expr(value)
		}
	}
}

fn expand_expr(namespaces: ~[namespace], expr: expr) -> expr
{
	match expr
	{
		constant_expr(iri_value(value)) =>
		{
			constant_expr(iri_value(expand_uri(namespaces, value)))
		}
		constant_expr(typed_value(value, kind)) =>
		{
			constant_expr(literal_to_object(@value, @expand_uri(namespaces, kind), @~""))
		}
		call_expr(fname, expressions) =>
		{
			call_expr(fname, vec::map(expressions, |e| {@expand_expr(namespaces, *e)}))
		}
		_ =>
		{
			expr
		}
	}
}

fn expand_pattern(namespaces: ~[namespace], pattern: pattern) -> pattern
{
	match pattern
	{
		constant(iri_value(value)) =>
		{
			constant(iri_value(expand_uri(namespaces, value)))
		}
		constant(typed_value(value, kind)) =>
		{
			constant(literal_to_object(@value, @expand_uri(namespaces, kind), @~""))
		}
		_ =>
		{
			pattern
		}
	}
}

fn expand_triple(namespaces: ~[namespace], tp: triple_pattern) -> triple_pattern
{
	{subject: expand_pattern(namespaces, tp.subject), predicate: expand_pattern(namespaces, tp.predicate), object: expand_pattern(namespaces, tp.object)}
}

fn expand(namespaces: ~[namespace], algebra: algebra) -> algebra
{
	match algebra
	{
		basic(pattern) =>
		{
			basic(expand_triple(namespaces, pattern))
		}
		group(terms) =>
		{
			group(vec::map(terms, |term| {@expand(namespaces, *term)}))
		}
		optional(term) =>
		{
			optional(@expand(namespaces, *term))
		}
		bind(expr, name) =>
		{
			bind(expand_expr(namespaces, expr), name)
		}
		filter(expr) =>
		{
			filter(expand_expr(namespaces, expr))
		}
	}
}

fn find_dupes(names: ~[~str]) -> ~[~str]
{
	let names = std::sort::merge_sort(|x, y| {*x <= *y}, names);	// TODO: probably dont want to de-reference the pointers
	
	let mut dupes = ~[];
	
	for vec::eachi(names)
	|i, name|
	{
		if i+1u < vec::len(names) && name == names[i+1u] && !vec::contains(dupes, name)
		{
			vec::push(dupes, name);
		}
	};
	
	return dupes;
}

pure fn is_hex(ch: char) -> bool
{
	return (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F');
}

pure fn is_langtag_prefix(ch: char) -> bool
{
	return char::is_alphabetic(ch);
}

pure fn is_langtag_suffix(ch: char) -> bool
{
	return is_langtag_prefix(ch) || char::is_digit(ch);
}

fn langtag() -> Parser<@~str>
{
	let at = "@".lit();												// '@'
	let prefix = match1(is_langtag_prefix);						// [a-zA-Z]+
	let suffix = do seq2("-".lit(), match1(is_langtag_suffix))
		|_l, name| {result::Ok(~"-" + *name)};
	let suffixes = suffix.r0();										// ('-' [a-zA-Z0-9]+)*
	
	do seq3(at, prefix, suffixes)
		|_l, p, s| {result::Ok(@(p + str::connect(*s, ~"")))}
}

// [150] ECHAR ::= '\' [tbnrf"']
fn is_escape_char(ch: char) -> bool
{
	option::is_some(str::find_char("tbnrf\"'", ch))	// input ends with EOT so we don't need a range check here
}

// [^<>"{}|^`\]-[#x00-#x20]
fn iri_char(ch: char) -> bool
{
	if option::is_none(str::find_char("^<>\"{}|^`\\", ch))
	{
		(ch as uint) > 0x20u
	}
	else
	{
		false
	}
}

// [^x\\\n\r]) | ECHAR	where x is ' or "
fn short_char(x: char, chars: @[char], index: uint) -> uint
{
	let mut i = index;
	loop
	{
		let ch = chars[i];
		if ch == EOT
		{
			return 0;
		}
		else if ch != x && ch != '\\' && ch != '\n' && ch != '\r'
		{
			i += 1;
		}
		else if ch == '\\' && is_escape_char(chars[i + 1u])
		{
			i += 2;
		}
		else
		{
			return i - index;
		}
	}
}

// ( "x" | "xx" )? ( [^x\] | ECHAR )	where x is ' or "
fn long_char(x: char, chars: @[char], index: uint) -> uint
{
	let mut i = index;
	loop
	{
		if chars[i] == x && chars[i+1] == x && chars[i+2] != x
		{
			i += 2;
		}
		else if chars[i] == x && chars[i+1] != x
		{
			i += 1;
		}
		
		let ch = chars[i];
		if ch != x && ch != '\\' && ch != EOT
		{
			i += 1;
		}
		else if ch == '\\' && is_escape_char(chars[i + 1])
		{
			i += 2;
		}
		else
		{
			return i - index;
		}
	}
}

// [154] PN_CHARS_BASE	::= [A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn pn_chars_base(chars: @[char], i: uint) -> uint
{
	if chars[i] >= 'A' && chars[i] <= 'Z'
	{
		1u
	}
	else if chars[i] >= 'a' && chars[i] <= 'z'
	{
		1u
	}
	else if chars[i] >= (0x00C0 as char) && chars[i] <= (0x00D6 as char)
	{
		1u
	}
	else if chars[i] >= (0x00D8 as char) && chars[i] <= (0x00F6 as char)
	{
		1u
	}
	else if chars[i] >= (0x00F8 as char) && chars[i] <= (0x02FF as char)
	{
		1u
	}
	else if chars[i] >= (0x0370 as char) && chars[i] <= (0x037D as char)
	{
		1u
	}
	else if chars[i] >= (0x037F as char) && chars[i] <= (0x1FFF as char)
	{
		1u
	}
	else if chars[i] >= (0x200C as char) && chars[i] <= (0x200D as char)
	{
		1u
	}
	else if chars[i] >= (0x2070 as char) && chars[i] <= (0x218F as char)
	{
		1u
	}
	else if chars[i] >= (0x2C00 as char) && chars[i] <= (0x2FEF as char)
	{
		1u
	}
	else if chars[i] >= (0x3001 as char) && chars[i] <= (0xD7FF as char)
	{
		1u
	}
	else if chars[i] >= (0xF900 as char) && chars[i] <= (0xFDCF as char)
	{
		1u
	}
	else if chars[i] >= (0xFDF0 as char) && chars[i] <= (0xFFFD as char)
	{
		1u
	}
	else if chars[i] >= (0x10000 as char) && chars[i] <= (0xEFFFF as char)
	{
		1u
	}
	else
	{
		0u
	}
}

// [155] PN_CHARS_U ::= PN_CHARS_BASE | '_'
fn pn_chars_u(chars: @[char], i: uint) -> uint
{
	let count = pn_chars_base(chars, i);
	if count > 0u
	{
		count
	}
	else if chars[i] == '_'
	{
		1u
	}
	else
	{
		0u
	}
}

// [157] PN_CHARS	::= PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
fn pn_chars(chars: @[char], i: uint) -> uint
{
	let count = pn_chars_u(chars, i);
	if count > 0u
	{
		count
	}
	else if chars[i] == '-'
	{
		1u
	}
	else if chars[i] >= '0' && chars[i] <= '9'
	{
		1u
	}
	else if chars[i] == (0xB7 as char)
	{
		1u
	}
	else if chars[i] >= (0x300 as char) && chars[i] <= (0x36F as char)
	{
		1u
	}
	else if chars[i] >= (0x203F as char) && chars[i] <= (0x2040 as char)
	{
		1u
	}
	else
	{
		0u
	}
}

fn pn_chars_or_dot(chars: @[char], index: uint) -> uint
{
	let mut i = index;
	loop
	{
		match pn_chars(chars, i)
		{
			0u =>
			{
				if chars[i] == '.'
				{
					i += 1;
				}
				else
				{
					return i - index;
				}
			}
			count =>
			{
				i += count;
			}
		}
	}
}

// [160] PLX ::= PERCENT | PN_LOCAL_ESC
fn plx(chars: @[char], i: uint) -> uint
{
	// [161] PERCENT ::= '%' HEX HEX
	if chars[i] == '%' && is_hex(chars[i+1u]) && is_hex(chars[i+2u])
	{
		3u
	}
	// [163] PN_LOCAL_ESC ::=  '\' ( '_' | '~' | '.' | '-' | '!' | '$' | '&' | "'" | '(' | ')' | '*' | '+' | ',' | ';' | '=' | ':' | '/' | '?' | '#' | '@' | '%' )
	else if chars[i] == '\\' && option::is_some(str::find_char("_~.-!$&'\"()*+,;=:/?#@%", chars[i+1u]))
	{
		2u
	}
	else
	{
		0u
	}
}

// PN_CHARS | '.' | PLX
fn pn_chars_or_dot_or_plx(chars: @[char], index: uint) -> uint
{
	let mut i = index;
	loop
	{
		match pn_chars(chars, i)
		{
			0u =>
			{
				if chars[i] == '.'
				{
					i += 1;
				}
				else
				{
					let n = plx(chars, i);
					if n > 0
					{
						i += n;
					}
					else
					{
						return i - index;
					}
				}
			}
			count =>
			{
				i += count;
			}
		}
	}
}

fn ws<T: copy owned>(parser: Parser<T>) -> Parser<T>
{
	|input: State|
	{
		do result::chain(parser(input))
		|pass|
		{
			let mut i = pass.new_state.index;
			let mut line = pass.new_state.line;
			loop
			{
				if input.text[i] == '\r' && input.text[i+1u] == '\n'
				{
					i += 2u;
					line += 1;
				}
				else if input.text[i] == '\n'
				{
					i += 1u;
					line += 1;
				}
				else if input.text[i] == '\r'
				{
					i += 1u;
					line += 1;
				}
				else if input.text[i] == '#'
				{
					while input.text[i] != '\r' && input.text[i] != '\n' && input.text[i] != '\x00'
					{
						i += 1u;
					}
				}
				else if input.text[i] == ' ' || input.text[i] == '\t'
				{
					i += 1u;
				}
				else
				{
					break;
				}
			}
			
			result::Ok({new_state: {index: i, line: line ,.. pass.new_state}, value: pass.value})
		}
	}
}

trait my_parser_trait<T: copy owned>
{
	fn ws() -> Parser<T>;
}

impl<T: copy owned> Parser<T> : my_parser_trait<T>
{
	fn ws() -> Parser<T>
	{
		ws(self)
	}
}

fn binary_expr(term: Parser<expr>, ops: ~[{oname: ~str, fname: ~str}]) -> Parser<expr>
{
	// Parser that returns which arm branched plus the value of the arm.
	let suffix = or_v(
		@vec::mapi(ops,
		|i, x|
		{
			seq2(x.oname.lit().ws(), term,
				|_o, r| {result::Ok((i, r))})
		}
	));
	
	let ops = copy ops;
	seq2(term, suffix.r0(),
	|b, r|
	{
		if vec::len(*r) == 0u
		{
			// If only one term matched then use that result.
			result::Ok(b)
		}
		else
		{
			// Otherwise we need to create call expressions for each pair of terms, from left to right.
			result::Ok(
				do vec::foldl(b, *r)
				|lhs, rhs|
				{
					let (i, Right) = rhs;
					call_expr(ops[i].fname, ~[@lhs, @Right])
				}
			)
		}
	}).err("")
}

// TODO: Hopefully rust will provide something better for converting and mixing ~str and & str.
// See https://github.com/mozilla/rust/issues/2992
fn unslice(s: &str) -> ~str
{
	s.slice(0, s.len())
}

fn built_in_call(Expression: Parser<expr>, Var: Parser<@~str>) -> Parser<expr>
{
	let var = seq3_ret1("(".lit().ws(), Var, ")".lit().ws());
	let nullary = seq2_ret1("(".lit().ws(), ")".lit().ws());
	let unary = seq3_ret1("(".lit().ws(), Expression, ")".lit().ws());
	let binary = seq5("(".lit().ws(), Expression, ",".lit().ws(), Expression, ")".lit().ws(), |_a0, a1, _a2, a3, _a4| {result::Ok(~[a1, a3])});
	let ternary = seq7("(".lit().ws(), Expression, ",".lit().ws(), Expression, ",".lit().ws(), Expression, ")".lit().ws(), |_a0, a1, _a2, a3, _a4, a5, _a6| {result::Ok(~[a1, a3, a5])});
	let variadic = seq3_ret1("(".lit().ws(), Expression.list(",".lit().ws()), ")".lit().ws());
	
	macro_rules! unary_fn (($name: expr) => { let n = $name.to_unique(); seq2(n.liti().ws(), unary, |_f, a| {result::Ok(call_expr(n + ~"_fn", ~[@a]))}) })
	macro_rules! binary_fn (($name: expr) => { let n = $name.to_unique(); seq2(n.liti().ws(), binary, |_f, a| {result::Ok(call_expr(n + ~"_fn", ~[@a[0], @a[1]]))}) })

	// [111] BuiltInCall ::= 
	or_v(@~[
		// 'STR' '(' Expression ')' 
		unary_fn!("str"),
		
		// |	'LANG' '(' Expression ')' 
		unary_fn!("lang"),
		
		// |	'LANGMATCHES' '(' Expression ',' Expression ')' 
		binary_fn!("langmatches"),
		
		// |	'DATATYPE' '(' Expression ')' 
		unary_fn!("datatype"),
		
		// |	'BOUND' '(' Var ')' 
		do seq2("BOUND".liti().ws(), var)	|_f, a0| {result::Ok(call_expr(~"bound_fn", ~[@variable_expr(*a0)]))},
		
		// |	'IRI' '(' Expression ')' 
		// |	'URI' '(' Expression ')' 
		// |	'BNODE' ( '(' Expression ')' | NIL)
		 
		// |	'RAND' NIL 
		do seq2("RAND".liti().ws(), nullary)	|_f, _a| {result::Ok(call_expr(~"rand_fn", ~[]))},
		
		// |	'ABS' '(' Expression ')' 
		unary_fn!("abs"),
		
		// |	'CEIL' '(' Expression ')' 
		unary_fn!("ceil"),
		
		// |	'FLOOR' '(' Expression ')' 
		unary_fn!("floor"),
		
		// |	'ROUND' '(' Expression ')' 
		unary_fn!("round"),
		
		// |	'CONCAT' ExpressionList 
		do seq2("CONCAT".liti().ws(), variadic)	|_f, a| {result::Ok(call_expr(~"concat_fn", vec::map(*a, {|e| @e})))},
		
		// |	SubstringExpression 
		do seq2("substr".liti().ws(), binary)	|_f, a| {result::Ok(call_expr(~"substr2_fn", ~[@a[0], @a[1]]))},
		do seq2("substr".liti().ws(), ternary)	|_f, a| {result::Ok(call_expr(~"substr3_fn", ~[@a[0], @a[1], @a[2]]))},
		
		// |	'STRLEN' '(' Expression ')' 
		unary_fn!("strlen"),
		
		// |	StrReplaceExpression 
		
		// |	'UCASE' '(' Expression ')' 
		unary_fn!("ucase"),
		
		// |	'LCASE' '(' Expression ')' 
		unary_fn!("lcase"),
		
		// |	'ENCODE_FOR_URI' '(' Expression ')' 
		unary_fn!("encode_for_uri"),
		
		// |	'CONTAINS' '(' Expression ',' Expression ')' 
		binary_fn!("contains"),
		
		// |	'STRSTARTS' '(' Expression ',' Expression ')' 
		binary_fn!("strstarts"),
		
		// |	'STRENDS' '(' Expression ',' Expression ')' 
		binary_fn!("strends"),
		
		// |	'STRBEFORE' '(' Expression ',' Expression ')' 
		binary_fn!("strbefore"),
		
		// |	'STRAFTER' '(' Expression ',' Expression ')' 
		binary_fn!("strafter"),
		
		// |	'YEAR' '(' Expression ')' 
		unary_fn!("year"),
		
		// |	'MONTH' '(' Expression ')' 
		unary_fn!("month"),
		
		// |	'DAY' '(' Expression ')' 
		unary_fn!("day"),
		
		// |	'HOURS' '(' Expression ')' 
		unary_fn!("hours"),
		
		// |	'MINUTES' '(' Expression ')' 
		unary_fn!("minutes"),
		
		// |	'SECONDS' '(' Expression ')' 
		unary_fn!("seconds"),
		
		// |	'TIMEZONE' '(' Expression ')' 
		
		// |	'TZ' '(' Expression ')' 
		unary_fn!("tz"),
		
		// |	'NOW' NIL 
		do seq2("NOW".liti().ws(), nullary)	|_f, _a| {result::Ok(call_expr(~"now_fn", ~[]))},
		
		// |	'MD5' '(' Expression ')' 
		// |	'SHA1' '(' Expression ')' 
		// |	'SHA256' '(' Expression ')' 
		// |	'SHA384' '(' Expression ')' 
		// |	'SHA512' '(' Expression ')' 
		
		// |	'COALESCE' ExpressionList 
		do seq2("COALESCE".liti().ws(), variadic)	|_f, a| {result::Ok(call_expr(~"coalesce_fn", vec::map(*a, {|e| @e})))},
		
		// |	'IF' '(' Expression ',' Expression ',' Expression ')' 
		do seq2("IF".liti().ws(), ternary)	|_f, a| {result::Ok(call_expr(~"if_fn", ~[@a[0], @a[1], @a[2]]))},
		
		// |	'STRLANG' '(' Expression ',' Expression ')' 
		binary_fn!("strlang"),
		
		// |	'STRDT' '(' Expression ',' Expression ')' 
		binary_fn!("strdt"),
		
		// |	'sameTerm' '(' Expression ',' Expression ')' 
		binary_fn!("sameterm"),
		
		// |	'isIRI' '(' Expression ')' 
		unary_fn!("isiri"),
		
		// |	'isURI' '(' Expression ')' 
		do seq2("isURI".liti().ws(), unary)	|_f, a| {result::Ok(call_expr(~"is_iri_fn", ~[@a]))},
		
		// |	'isBLANK' '(' Expression ')' 
		unary_fn!("isblank"),
		
		// |	'isLITERAL' '(' Expression ')' 
		unary_fn!("isliteral"),
		
		// |	'isNUMERIC' '(' Expression ')' 
		unary_fn!("isnumeric")
		
		// |	RegexExpression 
		// |	ExistsFunc 
		// |	NotExistsFunc
	]).err(~"built-in call")
}

// http://www.w3.org/TR/sparql11-query/#grammar
fn make_parser() -> Parser<selector>
{
	// [159] PN_LOCAL ::= (PN_CHARS_U | [0-9] | PLX)  ((PN_CHARS | '.' | PLX)* (PN_CHARS | PLX))? 		note that w3c had an error here (a stray > character at the end of the production)
	let pn_local_prefix = or_v(@~[
		scan(pn_chars_u),
		"0123456789".anyc().thene(|c| {ret(@str::from_char(c))}),
		scan(plx)
	]);
	let pn_local_suffix = do seq2(scan(pn_chars_or_dot_or_plx), scan(pn_chars).or(scan(plx)))
		|l, r| {result::Ok(@(*l + *r))};
	let PN_LOCAL = seq2(pn_local_prefix, optional_str(pn_local_suffix), |l, r| {result::Ok(@(*l + *r))});
	
	// [158] PN_PREFIX	::= PN_CHARS_BASE ((PN_CHARS | '.')* PN_CHARS)?
	let pname_ns_suffix = do seq2(scan(pn_chars_or_dot), scan(pn_chars))
		|l, r| {result::Ok(@(*l + *r))};
	
	let PN_PREFIX = seq2(scan(pn_chars_base), optional_str(pname_ns_suffix),
		|l, r| {result::Ok(@(*l + *r))});
	
	// [130] PNAME_NS	::= PN_PREFIX? ':'
	let PNAME_NS = seq2(optional_str(PN_PREFIX), ":".lit(), |l, r| {result::Ok(@(*l + *r))});
	
	// [131] PNAME_LN	::= PNAME_NS PN_LOCAL
	let PNAME_LN = seq2(PNAME_NS, PN_LOCAL, |l, r| {result::Ok(@(*l + *r))});
	
	// [149] STRING_LITERAL_LONG2 ::= '"""' (('"' | '""')? ([^"\] | ECHAR))* '"""'
	let STRING_LITERAL_LONG2 = seq3_ret1("\"\"\"".lit(), scan({|x, y| long_char('"', x, y)}), "\"\"\"".lit().ws());
	
	// [148] STRING_LITERAL_LONG1 ::= "'''" (("'" | "''")? ([^'\] | ECHAR))* "'''"
	let STRING_LITERAL_LONG1 = seq3_ret1("'''".lit(), scan({|x, y| long_char('\'', x, y)}), "'''".lit().ws());
	
	// [147] STRING_LITERAL2 ::= '"' (([^"\\\n\r]) | ECHAR)* '"'
	let STRING_LITERAL2 = seq3_ret1("\"".lit(), scan({|x, y| short_char('"', x, y)}), "\"".lit().ws());
	
	// [146] STRING_LITERAL1 ::= "'" (([^'\\\n\r]) | ECHAR)* "'"
	let STRING_LITERAL1 = seq3_ret1("'".lit(), scan({|x, y| short_char('\'', x, y)}), "'".lit().ws());
	
	// [136] INTEGER ::= [0-9]+
	let INTEGER = match1(is_digit).thene(|v| {ret(int_literal(v))}).ws();
	
	// [142] INTEGER_NEGATIVE ::= '-' INTEGER
	let INTEGER_NEGATIVE = seq2_ret_str("-".lit(), match1(is_digit)).thene(|v| {ret(int_literal(v))}).ws();
	
	// [139] INTEGER_POSITIVE ::= '+' INTEGER
	let INTEGER_POSITIVE = seq2_ret1("+".lit(), match1(is_digit)).thene(|v| {ret(int_literal(v))}).ws();
	
	// [137] DECIMAL ::= [0-9]* '.' [0-9]+
	let decimal_root = seq3_ret_str(match0(is_digit), ".".lit(), match1(is_digit)).ws();
	let DECIMAL = decimal_root.thene(|v| {ret(float_literal(v))});
	
	// [143] DECIMAL_NEGATIVE ::= '-' DECIMAL
	let DECIMAL_NEGATIVE = seq2_ret_str("-".lit(), decimal_root).thene(|v| {ret(float_literal(v))});
		
	// [140] DECIMAL_POSITIVE ::= '+' DECIMAL
	let DECIMAL_POSITIVE = seq2_ret1("+".lit(), decimal_root).thene(|v| {ret(float_literal(v))});
	
	// [145] EXPONENT ::= [eE] [+-]? [0-9]+
	let EXPONENT = seq3_ret_str("e".liti(), optional_str((("+".lit()).or("-".lit()))), match1(is_digit));
	
	// [138] DOUBLE ::= [0-9]+ '.' [0-9]* EXPONENT | 
	//                           '.' ([0-9])+ EXPONENT | 
	//                           ([0-9])+ EXPONENT
	let double1 = seq4_ret_str(match1(is_digit), ".".lit(), match0(is_digit), EXPONENT);
	let double2 = seq3_ret_str(".".lit(), match1(is_digit), EXPONENT);
	let double3 = seq2_ret_str(match1(is_digit), EXPONENT);
	
	let double_root = or_v(@~[double1, double2, double3]).ws();
	let DOUBLE = double_root.thene(|v| {ret(float_literal(v))});
	
	// [144] DOUBLE_NEGATIVE ::= '-' DOUBLE
	let DOUBLE_NEGATIVE = seq2_ret_str("-".lit(), double_root).thene(|v| {ret(float_literal(v))});
	
	// [141] DOUBLE_POSITIVE ::= '+' DOUBLE
	let DOUBLE_POSITIVE = seq2_ret1("+".lit(), double_root).thene(|v| {ret(float_literal(v))});
	
	// [135] LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
	let LANGTAG = langtag();
	
	// [129] IRI_REF	 ::= '<' ([^<>"{}|^`\]-[#x00-#x20])* '>'
	let IRI_REF = seq3_ret1("<".lit(), match0(iri_char), ">".lit().ws()).note(~"IRI_REF");
	
	// [127] PrefixedName ::= PNAME_LN | PNAME_NS
	let PrefixedName = (PNAME_LN.or(PNAME_NS)).note(~"prefixedname").ws();
	
	// [126] IRIref ::= IRI_REF | PrefixedName
	let IRIref = IRI_REF.or(PrefixedName);
	
	// [125] String ::= STRING_LITERAL1 | STRING_LITERAL2 | STRING_LITERAL_LONG1 | STRING_LITERAL_LONG2
	let String = or_v(@~[STRING_LITERAL_LONG1, STRING_LITERAL_LONG2, STRING_LITERAL1, STRING_LITERAL2]).note(~"string");
	
	// [124] BooleanLiteral	::= 'true' | 'false'
	let BooleanLiteral = ("true".lit()).or("false".lit()).thene({|v| ret(bool_literal(v))}).ws().note(~"boolean");
	
	// [121] NumericLiteralUnsigned ::= INTEGER | DECIMAL | DOUBLE
	let NumericLiteralUnsigned = or_v(@~[DOUBLE, DECIMAL, INTEGER]).note(~"number");
	
	// [122] NumericLiteralPositive ::= INTEGER_POSITIVE |	DECIMAL_POSITIVE | DOUBLE_POSITIVE
	let NumericLiteralPositive = or_v(@~[DOUBLE_POSITIVE, DECIMAL_POSITIVE, INTEGER_POSITIVE]);

	// [123] NumericLiteralNegative ::= INTEGER_NEGATIVE |	DECIMAL_NEGATIVE |	DOUBLE_NEGATIVE
	let NumericLiteralNegative = or_v(@~[DOUBLE_NEGATIVE, DECIMAL_NEGATIVE, INTEGER_NEGATIVE]);
	
	// [120] NumericLiteral	::= NumericLiteralUnsigned | NumericLiteralPositive | NumericLiteralNegative
	let NumericLiteral = or_v(@~[NumericLiteralPositive, NumericLiteralNegative, NumericLiteralUnsigned]);
	
	// [119] RDFLiteral ::= String (LANGTAG | ('^^' IRIref))?
	let RDFLiteral1 = String.thene({|v| ret(string_literal(v, @~""))}); 
	
	let RDFLiteral2 = do seq2(String, LANGTAG)
		|v, l| {result::Ok(string_literal(v, l))};
	
	let RDFLiteral3 = do seq3(String, "^^".lit(), IRIref)
		|v, _m, t| {result::Ok(typed_literal(v, t))};
	
	let RDFLiteral = or_v(@~[RDFLiteral3, RDFLiteral2, RDFLiteral1]);
	
	// [99] GraphTerm ::= IRIref | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | NIL
	let GraphTerm = or_v(@~[
		RDFLiteral,
		IRIref.thene({|v| ret(iri_literal(v))}),	// TODO: support BlankNode and NIL
		do NumericLiteral.thene |v| {ret(constant(v))},
		BooleanLiteral
	]);
	
	// [156] VARNAME ::= (PN_CHARS_U | [0-9]) (PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040])*
	let VARNAME = identifier().ws();
	
	// [133] VAR1 ::= '?' VARNAME
	let VAR1 = seq2_ret1("?".lit().ws(), VARNAME).note(~"VAR1");
	
	// [67] ArgList ::= NIL | '(' 'DISTINCT'? Expression ( ',' Expression )* ')'
	let Expression_ptr = @mut ret(constant_expr(unbound_value(~"foo")));
	let Expression_ref = forward_ref(Expression_ptr);
	
	let ArgList = seq3_ret1("(".lit().ws(), Expression_ref.list(",".lit().ws()), ")".lit().ws());
	
	// [118] IRIrefOrFunction ::= IRIref ArgList?
	let IRIrefOrFunction1 = do seq2(IRIref, ArgList)
		|i, a| {result::Ok(extension_expr(*i, vec::map(*a, |x| {@x})))};
	let IRIrefOrFunction2 = do IRIref.thene |v| {ret(constant_expr(iri_value(*v)))};
	let IRIrefOrFunction = (IRIrefOrFunction1.or(IRIrefOrFunction2));
	
	// [98] Var ::= VAR1 | VAR2
	let Var = VAR1;
	
	// [111] BuiltInCall
	let BuiltInCall = built_in_call(Expression_ref, Var);
	
	// [109] PrimaryExpression ::= BrackettedExpression | BuiltInCall | IRIrefOrFunction | RDFLiteral | NumericLiteral | BooleanLiteral | Var | Aggregate
	let BrackettedExpression_ptr = @mut ret(constant_expr(unbound_value(~"foo")));
	let BrackettedExpression_ref = forward_ref(BrackettedExpression_ptr);
	
	let PrimaryExpression = or_v(@~[
		BrackettedExpression_ref,
		IRIrefOrFunction,
		BuiltInCall,
		do RDFLiteral.thene |v| {ret(pattern_to_expr(v))},
		do NumericLiteral.thene |v| {ret(constant_expr(v))},
		do Var.thene |v| {ret(variable_expr(*v))},
		do BooleanLiteral.thene |v| {ret(pattern_to_expr(v))}
		]);
		
	// [108] UnaryExpression ::= '!' PrimaryExpression | '+' PrimaryExpression | '-' PrimaryExpression | PrimaryExpression
	let UnaryExpression = or_v(@~[
		do seq2_ret1("!".lit().ws(), PrimaryExpression).thene |term| {ret(call_expr(~"op_not", ~[@term]))},
		seq2_ret1("+".lit().ws(), PrimaryExpression),
		do seq2_ret1("-".lit().ws(), PrimaryExpression).thene |term| {ret(call_expr(~"op_unary_minus", ~[@term]))},
		PrimaryExpression
		]);
	
	// [107] MultiplicativeExpression ::= UnaryExpression ('*' UnaryExpression | '/' UnaryExpression)*
	let MultiplicativeExpression = binary_expr(UnaryExpression, ~[{oname: ~"*", fname: ~"op_multiply"}, {oname: ~"/", fname: ~"op_divide"}]);
	
	// [106] AdditiveExpression ::= MultiplicativeExpression (
	//                                                 '+' MultiplicativeExpression | '-' MultiplicativeExpression | 
	//                                                 (NumericLiteralPositive | NumericLiteralNegative ) (('*' UnaryExpression) | ('/' UnaryExpression))?
	//                                           )*
	let AdditiveExpression = binary_expr(MultiplicativeExpression, ~[{oname:~"+", fname: ~"op_add"}, {oname: ~"-", fname: ~"op_subtract"}]);
	
	// [105] NumericExpression ::= AdditiveExpression
	let NumericExpression = AdditiveExpression;
	
	// [104] RelationalExpression ::= NumericExpression ( '=' NumericExpression | '!=' NumericExpression | 
	//                                                                           '<' NumericExpression | '>' NumericExpression | 
	//                                                                           '<=' NumericExpression | '>=' NumericExpression | 
	//                                                                           'IN' ExpressionList | 'NOT' 'IN' ExpressionList )?
	let RelationalExpression = or_v(@~[
		do seq3(NumericExpression, "=".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_equals", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, "!=".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_not_equals", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, "<".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_less_than", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, ">".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_greater_than", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, "<=".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_less_than_or_equal", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, ">=".lit().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"op_greater_than_or_equal", ~[@lhs, @rhs]))},
		do seq3(NumericExpression, "IN".liti().ws(), NumericExpression)
			|lhs, _op, rhs| {result::Ok(call_expr(~"in_op", ~[@lhs, @rhs]))},
		do seq4(NumericExpression, "NOT".liti().ws(), "IN".liti().ws(), NumericExpression)
			|lhs, _o1, _o2, rhs| {result::Ok(call_expr(~"not_in_op", ~[@lhs, @rhs]))},
		NumericExpression
	]);
	
	// [103] ValueLogical ::= RelationalExpression
	let ValueLogical = RelationalExpression;
	
	// [102] ConditionalAndExpression ::= ValueLogical ( '&&' ValueLogical )*
	let ConditionalAndExpression = binary_expr(ValueLogical, ~[{oname: ~"&&", fname: ~"op_and"}]);
	
	// [101] ConditionalOrExpression ::= ConditionalAndExpression ( '||' ConditionalAndExpression )*
	let ConditionalOrExpression = binary_expr(ConditionalAndExpression, ~[{oname: ~"||", fname: ~"op_or"}]);
	
	// [100] Expression ::= ConditionalOrExpression
	let Expression = ConditionalOrExpression;
	*Expression_ptr = Expression;
	
	// [110] BrackettedExpression ::= '(' Expression ')'
	let BrackettedExpression = seq3_ret1("(".lit().ws(), Expression, ")".lit().ws());
	*BrackettedExpression_ptr = BrackettedExpression;
	
	// [96] VarOrTerm ::= Var | GraphTerm
	let VarOrTerm = (Var.thene(|v| {ret(variable((*v)))})).or(GraphTerm);
	
	// [95] GraphNode ::= VarOrTerm | TriplesNode
	let GraphNode = VarOrTerm;
	
	// [88] PathPrimary ::= IRIref | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
	let PathPrimary = IRIref.thene({|v| ret(iri_literal(v))});
	
	// [85] PathElt ::= PathPrimary PathMod?
	let PathElt = PathPrimary;
	
	// [86] PathEltOrInverse ::= PathElt | '^' PathElt
	let PathEltOrInverse = PathElt;
	
	// [84] PathSequence ::= PathEltOrInverse ( '/' PathEltOrInverse )*
	let PathSequence = PathEltOrInverse;
	
	// [83] PathAlternative	::= PathSequence ( '|' PathSequence )*
	let PathAlternative = PathSequence;
	
	// [82] Path ::= PathAlternative
	let Path = PathAlternative;
	
	// [81] VerbSimple ::= Var
	let VerbSimple = Var.thene(|v| {ret(variable((*v)))});
	
	// [75] Object ::= GraphNode
	let Object = GraphNode;
	
	// [74] ObjectList ::= Object ( ',' Object )*
	let ObjectList = Object;
	
	// [80] VerbPath	::= Path
	let VerbPath = Path;
	
	// [78] PropertyListNotEmptyPath	::= (VerbPath | VerbSimple) ObjectList ( ';' ( ( VerbPath | VerbSimple ) ObjectList )? )*
	let PropertyListNotEmptyPath= do seq2(VerbPath.or(VerbSimple), ObjectList)
		|prop, object| {result::Ok([prop, object])};
		
	// [77] TriplesSameSubjectPath ::= VarOrTerm PropertyListNotEmptyPath | TriplesNode PropertyListPath 
	let TriplesSameSubjectPath = seq2(VarOrTerm, PropertyListNotEmptyPath,
		|subject, e| {result::Ok({subject: subject, predicate: e[0], object: e[1]})}).note(~"TriplesSameSubjectPath");
		
	// [65] Constraint ::= BrackettedExpression | BuiltInCall | FunctionCall
	let Constraint = or_v(@~[BrackettedExpression, BuiltInCall]).note(~"Constraint");
	
	// [64] Filter ::= 'FILTER' Constraint
	let Filter = seq2_ret1("FILTER".liti().ws(), Constraint).thene(|v| {ret(filter(v))}).note(~"filter");
	
	// [61] Bind ::= 'BIND' '(' Expression 'AS' Var ')'
	let Bind = seq6("BIND".liti().ws(), "(".lit().ws(), Expression, "AS".liti().ws(), Var, ")".lit().ws(),
		|_b, _p, e, _a, v, _q| {result::Ok(bind(e, *v))}).note(~"bind");
	
	// [58] OptionalGraphPattern ::= 'OPTIONAL' GroupGraphPattern
	let GroupGraphPattern_ptr = @mut ret(group(~[]));
	let GroupGraphPattern_ref = forward_ref(GroupGraphPattern_ptr);
	
	let OptionalGraphPattern = do seq2("OPTIONAL".liti().ws(), GroupGraphPattern_ref)
		|_o, a| {result::Ok(optional(@a))};
	
	// [57] GraphPatternNotTriples ::= GroupOrUnionGraphPattern | OptionalGraphPattern | MinusGraphPattern | 
	//                                                GraphGraphPattern | ServiceGraphPattern | Filter | Bind
	let GraphPatternNotTriples = or_v(@~[OptionalGraphPattern, Filter, Bind]).note(~"GraphPatternNotTriples");
	
	// [56] TriplesBlock ::= TriplesSameSubjectPath ('.' TriplesBlock?)?
	let TriplesBlock = seq2(TriplesSameSubjectPath.list(".".lit().ws()), ".".lit().ws().optional(),
		|patterns, _r|
		{
			if vec::len(*patterns) == 1
			{
				result::Ok(basic(patterns[0]))
			}
			else
			{
				result::Ok(group(vec::map(*patterns, {|p| @basic(p)})))
			}
		}).note(~"TriplesBlock");
	
	// [55] GroupGraphPatternSub ::= TriplesBlock? (GraphPatternNotTriples '.'? TriplesBlock?)*
	let ggps_suffix = do seq3(GraphPatternNotTriples, ".".lit().ws().optional(), TriplesBlock.optional())
		|gpnt, _d, tb|
		{
			if option::is_some(tb)
			{
				result::Ok(group(~[@gpnt, @(tb.get())]))
			}
			else
			{
				result::Ok(gpnt)
			}
		};
	let GroupGraphPatternSub = do seq2(TriplesBlock.optional(), ggps_suffix.r0())
		|tb, gp|
		{
			let patterns =
				if option::is_some(tb)
				{
					~[tb.get()] + *gp
				}
				else
				{
					*gp
				};
			
			if vec::len(patterns) == 1
			{
				result::Ok(patterns[0])
			}
			else
			{
				result::Ok(group(vec::map(patterns, {|p| @p})))
			}
		};
		
	// [54] GroupGraphPattern ::= '{' (SubSelect | GroupGraphPatternSub) '}'
	let GroupGraphPattern = seq3_ret1("{".lit().ws(), GroupGraphPatternSub, "}".lit().ws());
	*GroupGraphPattern_ptr = GroupGraphPattern.note(~"GroupGraphPattern");
	
	// [26] LimitClause ::= 'LIMIT' INTEGER
	let LimitClause = do seq2_ret1("LIMIT".liti().ws(), INTEGER).thene
		|x| {match x {int_value(n)  => ret(n as uint), _  => fail(~"Somehow INTEGER didn't return an int_value")}};
	
	// [25] LimitOffsetClauses	::= LimitClause OffsetClause? | OffsetClause LimitClause?
	let LimitOffsetClauses = LimitClause;
	
	// [24] OrderCondition ::= (('ASC' | 'DESC') BrackettedExpression) | (Constraint | Var)
	let OrderCondition1 = do seq2_ret1("ASC".liti().ws(), BrackettedExpression).thene |v| {ret(call_expr(~"!asc", ~[@v]))};
	let OrderCondition2 = do seq2_ret1("DESC".liti().ws(), BrackettedExpression).thene |v| {ret(call_expr(~"!desc", ~[@v]))};
	let OrderCondition3 = Constraint.or(do Var.thene |v| {ret(variable_expr(*v))});
	let OrderCondition = or_v(@~[OrderCondition1, OrderCondition2, OrderCondition3]);
	
	// [23] OrderClause ::= 'ORDER' 'BY' OrderCondition+
	let OrderClause = seq3_ret2("ORDER".liti().ws(), "BY".liti().ws(), OrderCondition.r1()).note(~"OrderClause");
	
	// [18] SolutionModifier ::= GroupClause? HavingClause? OrderClause? LimitOffsetClauses?
	let SolutionModifier = do seq2(OrderClause.optional(), LimitOffsetClauses.optional())
		|a, b| {result::Ok({order_by: a, limit: b})};
	
	// [17] WhereClause ::= 'WHERE'? GroupGraphPattern
	let WhereClause = seq2_ret1(("WHERE".liti().ws()).optional(), GroupGraphPattern).note(~"WhereClause"); 
	
	// [9] SelectClause ::= 'SELECT' ('DISTINCT' | 'REDUCED')? ((Var | ('(' Expression 'AS' Var ')'))+ | '*')
	let select_suffix = or_v(@~[
		(Var.thene(|v| {ret(variable((*v)))})).r1(),
		"*".lit().ws().thene(|_x| {ret(@~[variable(~"*")])})]);
		
	let select_mid = ("DISTINCT".liti()).or("REDUCED".liti()).ws().optional();
		
	let SelectClause = seq3("SELECT".liti().ws(), select_mid, select_suffix,
		|_a, b, c| {result::Ok((option::is_some(b), *c))}).note(~"SelectClause");
		
	// [7] SelectQuery ::= SelectClause DatasetClause* WhereClause SolutionModifier
	let SelectQuery = do seq3(SelectClause, WhereClause, SolutionModifier)
		|patterns, algebra, modifiers| {result::Ok((patterns, algebra, modifiers))};
		
	// [6] PrefixDecl ::= 'PREFIX' PNAME_NS IRI_REF
	let PrefixDecl = do seq3("PREFIX".liti().ws(), PNAME_NS.ws(), IRI_REF)
		|_p, ns, r| {result::Ok({prefix: str::slice(*ns, 0u, str::len(*ns)-1u), path: *r})};
	
	// [4] Prologue ::= (BaseDecl | PrefixDecl)*
	let Prologue = PrefixDecl.r0();
	
	// [2] Query ::= Prologue (SelectQuery | ConstructQuery | DescribeQuery | AskQuery) BindingsClause
	let Query = do seq2(Prologue, SelectQuery)
		|p, s| {build_parser(*p, s)};
	
	// [1] QueryUnit ::= Query
	let QueryUnit = Query.everything(ret(0).ws());
	
	return QueryUnit;
}

type SolutionModifiers = {order_by: Option<@~[expr]>, limit: Option<uint>};

// namespaces are from the PREFIX clauses
// patterns are from the SELECT clause
// algebra is from the WHERE clause
fn build_parser(namespaces: ~[namespace], query: ((bool, ~[pattern]), algebra, SolutionModifiers)) -> result::Result<selector, @~str>
{
	let ((distinct, patterns), algebra, modifiers) = query;
	
	let variables = do vec::filter(patterns) |p| {match p {variable(_l)  => true, _  => false}};
	let names = do vec::map(variables) |p| {match p {variable(n)  => n, _  => fail}};
	
	let order_by = match modifiers.order_by {option::Some(x)  => x, option::None  => @~[]};
	
	let dupes = find_dupes(names);
	if vec::is_empty(dupes)
	{
		// eval will set namespaces and extensions
		if vec::is_not_empty(namespaces)
		{
			let context = {namespaces: ~[], extensions: @str_hash(), algebra: expand(namespaces, algebra), order_by: *order_by, distinct: distinct, limit: modifiers.limit, rng: rand::Rng(), timestamp: time::now()};
			result::Ok(eval(names, context))
		}
		else
		{
			let context = {namespaces: ~[], extensions: @str_hash(), algebra: algebra, order_by: *order_by, distinct: distinct, limit: modifiers.limit, rng: rand::Rng(), timestamp: time::now()};
			result::Ok(eval(names, context))
		}
	}
	else
	{
		result::Err(@fmt!("Select clause has duplicates: %s", str::connect(dupes, ~" ")))
	}
}

/// Returns either a function capable of matching triples or a parse error.
/// 
/// Expr can be a subset of http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"SPARQL\".
fn compile(expr: ~str) -> result::Result<selector, ~str>
{
	let parser = make_parser();
	do result::chain_err(parser.parse(@~"sparql", expr))
	|err|
	{
		result::Err(fmt!("%s on line %? col %?", *err.mesg, err.line, err.col))
	}
}
