import rparse::*;
import query::*;

type triple_pattern = {subject: pattern, predicate: pattern, object: pattern};

enum algebra
{
	basic(triple_pattern),
	group([@algebra]),
	optional(@algebra)
}

fn expand_pattern(namespaces: [namespace], pattern: pattern) -> pattern
{
	alt pattern
	{
		constant({value: value, kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""})
		{
			constant({value: expand_uri(namespaces, value), kind: "http://www.w3.org/2001/XMLSchema#anyURI", lang: ""})
		}
		_
		{
			pattern
		}
	}
}

fn expand_triple(namespaces: [namespace], tp: triple_pattern) -> triple_pattern
{
	{subject: expand_pattern(namespaces, tp.subject), predicate: expand_pattern(namespaces, tp.predicate), object: expand_pattern(namespaces, tp.object)}
}

fn expand(namespaces: [namespace], algebra: algebra) -> algebra
{
	alt algebra
	{
		basic(pattern)
		{
			basic(expand_triple(namespaces, pattern))
		}
		group(terms)
		{
			group(vec::map(terms, {|term| @expand(namespaces, *term)}))
		}
		optional(term)
		{
			optional(@expand(namespaces, *term))
		}
	}
}

fn find_dupes(names: [str]) -> [str]
{
	let names = std::sort::merge_sort({|x, y| x <= y}, names);
	
	let mut dupes = [];
	
	for vec::eachi(names)
	{|i, name|
		if i+1u < vec::len(names) && name == names[i+1u] && !vec::contains(dupes, name)
		{
			vec::push(dupes, name);
		}
	};
	
	ret dupes;
}

pure fn is_hex(ch: char) -> bool
{
	ret (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F');
}

pure fn is_langtag_prefix(ch: char) -> bool
{
	ret is_alpha(ch);
}

pure fn is_langtag_suffix(ch: char) -> bool
{
	ret is_langtag_prefix(ch) || is_digit(ch);
}

fn langtag() -> parser<str>
{
	let at = "@".lit();													// '@'
	let prefix = match1(is_langtag_prefix).tag("Expected language");	// [a-zA-Z]+
	let suffix = seq2("-".lit(), match1(is_langtag_suffix).tag("Expected language"))
		{|_l, name| result::ok("-" + name)};
	let suffixes = suffix.r0();											// ('-' [a-zA-Z0-9]+)*
	
	seq3(at, prefix, suffixes)
		{|_l, p, s| result::ok(p + str::connect(s, ""))}
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
fn short_char(x: char, chars: [char], i:uint) -> uint
{
	let ch = chars[i];
	if ch != x && ch != '\\' && ch != '\n' && ch != '\r'
	{
		1u
	}
	else if ch == '\\' && is_escape_char(chars[i + 1u])
	{
		2u
	}
	else
	{
		0u
	}
}

// ( "x" | "xx" )? ( [^x\] | ECHAR )	where x is ' or "
fn long_char(x: char, chars: [char], i:uint) -> uint
{
	let delta = if chars[i] == x
	{
		if chars[i+1u] == x
		{
			2u
		}
		else
		{
			1u
		}
	}
	else
	{
		0u
	};
	
	let ch = chars[i + delta];
	if ch != x && ch != '\\'
	{
		delta + 1u
	}
	else if ch == '\\' && is_escape_char(chars[i + delta + 1u])
	{
		delta + 2u
	}
	else
	{
		0u
	}
}

// [154] PN_CHARS_BASE	::= [A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn pn_chars_base(chars: [char], i: uint) -> uint
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
fn pn_chars_u(chars: [char], i: uint) -> uint
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
fn pn_chars(chars: [char], i: uint) -> uint
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

fn pn_chars_or_dot(chars: [char], index: uint) -> uint
{
	alt pn_chars(chars, index)
	{
		0u
		{
			if chars[index] == '.'
			{
				1u
			}
			else
			{
				0u
			}
		}
		count
		{
			count
		}
	}
}

// [160] PLX ::= PERCENT | PN_LOCAL_ESC
fn plx(chars: [char], i: uint) -> uint
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
fn pn_chars_or_dot_or_plx(chars: [char], i: uint) -> uint
{
	alt pn_chars(chars, i)
	{
		0u
		{
			if chars[i] == '.'
			{
				1u
			}
			else
			{
				plx(chars, i)
			}
		}
		count
		{
			count
		}
	}
}

fn ws<T: copy>(parser: parser<T>) -> parser<T>
{
	// It would be simpler to write this with scan0, but scan0 is relatively inefficient
	// and ws is typically called a lot.
	{|input: state|
		result::chain(parser(input))
		{|pass|
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
			
			log_ok("ws", input, {new_state: {index: i, line: line with pass.new_state}, value: pass.value})
		}
	}
}

impl my_parser_methods<T: copy> for parser<T>
{
	fn ws() -> parser<T>
	{
		ws(self)
	}
}

fn typed_literal(value: str, kind: str) -> pattern
{
	constant({value: value, kind: kind, lang: ""})
}

fn string_literal(value: str, lang: str) -> pattern
{
	constant({value: value, kind: "http://www.w3.org/2001/XMLSchema#string", lang: lang})
}

// http://www.w3.org/TR/sparql11-query/#grammar
// TODO: remove annotate calls
fn make_parser() -> parser<selector>
{
	// [159] PN_LOCAL ::= (PN_CHARS_U | [0-9] | PLX)  ((PN_CHARS | '.' | PLX)* (PN_CHARS | PLX))? 		note that w3c had an error here (a stray > character at the end of the production)
	let pn_local_prefix = or_v([
		scan(pn_chars_u),
		"0123456789".anyc().thene({|c| return(str::from_char(c))}),
		scan(plx)
	]);
	let pn_local_suffix = seq2(scan0(pn_chars_or_dot_or_plx), scan(pn_chars).or(scan(plx)))
		{|l, r| result::ok(l + r)};
	let PN_LOCAL = seq2(pn_local_prefix, optional_str(pn_local_suffix))
		{|l, r| result::ok(l + r)};
	
	// [158] PN_PREFIX	::= PN_CHARS_BASE ((PN_CHARS | '.')* PN_CHARS)?
	let pname_ns_suffix = seq2(scan(pn_chars_or_dot), scan(pn_chars))
		{|l, r| result::ok(l + r)};
	
	let PN_PREFIX = seq2(scan(pn_chars_base), optional_str(pname_ns_suffix))
		{|l, r| result::ok(l + r)};
	
	// [130] PNAME_NS	::= PN_PREFIX? ':'
	let PNAME_NS = seq2(optional_str(PN_PREFIX), ":".lit())
		{|l, r| result::ok(l + r)};
	
	// [131] PNAME_LN	::= PNAME_NS PN_LOCAL
	let PNAME_LN = seq2(PNAME_NS, PN_LOCAL)
		{|l, r| result::ok(l + r)};
	
	// [149] STRING_LITERAL_LONG2 ::= '"""' ( ( '"' | '""' )? ( [^"\] | ECHAR ) )* '"""'
	let STRING_LITERAL_LONG2 = seq3_ret1("\"\"\"".lit(), scan0(bind long_char('"', _, _)), "\"\"\"".lit().ws());
	
	// [148] STRING_LITERAL_LONG1 ::= "'''" ( ( "'" | "''" )? ( [^'\] | ECHAR ) )* "'''"
	let STRING_LITERAL_LONG1 = seq3_ret1("'''".lit(), scan0(bind long_char('\'', _, _)), "'''".lit().ws());
	
	// [147] STRING_LITERAL2 ::= '"' ( ([^"\\\n\r]) | ECHAR )* '"'
	let STRING_LITERAL2 = seq3_ret1("\"".lit(), scan0(bind short_char('"', _, _)), "\"".lit().ws());
	
	// [146] STRING_LITERAL1 ::= "'" ( ([^'\\\n\r]) | ECHAR )* "'"
	let STRING_LITERAL1 = seq3_ret1("'".lit(), scan0(bind short_char('\'', _, _)), "'".lit().ws());
	
	// [136] INTEGER ::= [0-9]+
	let INTEGER = match1(is_digit).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#integer"))}).ws();
	
	// [142] INTEGER_NEGATIVE ::= '-' INTEGER
	let INTEGER_NEGATIVE = seq2_ret_str("-".lit(), match1(is_digit)).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#integer"))}).ws();
	
	// [139] INTEGER_POSITIVE ::= '+' INTEGER
	let INTEGER_POSITIVE = seq2_ret1("+".lit(), match1(is_digit)).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#integer"))}).ws();
	
	// [137] DECIMAL ::= [0-9]* '.' [0-9]+
	let decimal_root = seq3_ret_str(match0(is_digit), ".".lit(), match1(is_digit)).ws();
	let DECIMAL = decimal_root.thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
	
	// [143] DECIMAL_NEGATIVE ::= '-' DECIMAL
	let DECIMAL_NEGATIVE = seq2_ret_str("-".lit(), decimal_root).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
		
	// [140] DECIMAL_POSITIVE ::= '+' DECIMAL
	let DECIMAL_POSITIVE = seq2_ret1("+".lit(), decimal_root).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
	
	// [145] EXPONENT ::= [eE] [+-]? [0-9]+
	let EXPONENT = seq3_ret_str("e".liti(), optional_str((("+".lit()).or("-".lit()))), match1(is_digit));
	
	// [138] DOUBLE ::= [0-9]+ '.' [0-9]* EXPONENT | 
	//                           '.' ([0-9])+ EXPONENT | 
	//                           ([0-9])+ EXPONENT
	let double1 = seq4_ret_str(match1(is_digit), ".".lit(), match0(is_digit), EXPONENT);
	let double2 = seq3_ret_str(".".lit(), match1(is_digit), EXPONENT);
	let double3 = seq2_ret_str(match1(is_digit), EXPONENT);
	
	let double_root = or_v([double1, double2, double3]).ws();
	let DOUBLE = double_root.thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
	
	// [144] DOUBLE_NEGATIVE ::= '-' DOUBLE
	let DOUBLE_NEGATIVE = seq2_ret_str("-".lit(), double_root).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
	
	// [141] DOUBLE_POSITIVE ::= '+' DOUBLE
	let DOUBLE_POSITIVE = seq2_ret1("+".lit(), double_root).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#double"))});
	
	// [135] LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
	let LANGTAG = langtag();
	
	// [129] IRI_REF	 ::= '<' ([^<>"{}|^`\]-[#x00-#x20])* '>'
	let IRI_REF = seq3_ret1("<".lit(), match0(iri_char), ">".lit().ws()).annotate("IRI_REF");
	
	// [127] PrefixedName ::= PNAME_LN | PNAME_NS
	let PrefixedName = (PNAME_LN.or(PNAME_NS)).ws();
	
	// [126] IRIref ::= IRI_REF | PrefixedName
	let IRIref = IRI_REF.or(PrefixedName.annotate("prefixedname"));
	
	// [125] String ::= STRING_LITERAL1 | STRING_LITERAL2 | STRING_LITERAL_LONG1 | STRING_LITERAL_LONG2
	let String = or_v([STRING_LITERAL_LONG1, STRING_LITERAL_LONG2, STRING_LITERAL1, STRING_LITERAL2]);
	
	// [124] BooleanLiteral	::= 'true' | 'false'
	let BooleanLiteral = ("true".lit()).or("false".lit()).thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#boolean"))}).ws();
	
	// [121] NumericLiteralUnsigned ::= INTEGER | DECIMAL | DOUBLE
	let NumericLiteralUnsigned = or_v([DOUBLE, DECIMAL, INTEGER]);
	
	// [122] NumericLiteralPositive ::= INTEGER_POSITIVE |	DECIMAL_POSITIVE |	DOUBLE_POSITIVE
	let NumericLiteralPositive = or_v([DOUBLE_POSITIVE, DECIMAL_POSITIVE, INTEGER_POSITIVE]);

	// [123] NumericLiteralNegative ::= INTEGER_NEGATIVE |	DECIMAL_NEGATIVE |	DOUBLE_NEGATIVE
	let NumericLiteralNegative = or_v([DOUBLE_NEGATIVE, DECIMAL_NEGATIVE, INTEGER_NEGATIVE]);
	
	// [120] NumericLiteral	::= NumericLiteralUnsigned | NumericLiteralPositive | NumericLiteralNegative
	let NumericLiteral = or_v([NumericLiteralPositive, NumericLiteralNegative, NumericLiteralUnsigned]);
	
	// [119] RDFLiteral ::= String ( LANGTAG | ( '^^' IRIref ) )?
	let RDFLiteral1 = String.thene({|v| return(string_literal(v, ""))});
	
	let RDFLiteral2 = seq2(String, LANGTAG)
		{|v, l| result::ok(string_literal(v, l))};
	
	let RDFLiteral3 = seq3(String, "^^".lit(), IRIref)
		{|v, _m, t| result::ok(typed_literal(v, t))};
	
	let RDFLiteral = or_v([RDFLiteral3, RDFLiteral2, RDFLiteral1]);
	
	// [99] GraphTerm	::= IRIref | RDFLiteral | NumericLiteral | BooleanLiteral |	BlankNode |	NIL
	let GraphTerm = or_v([
		RDFLiteral.annotate("RDFLiteral"),
		IRIref.annotate("IRIref").thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#anyURI"))}),
		NumericLiteral,
		BooleanLiteral
	]);
	
	// [156] VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*
	let VARNAME = identifier().ws();
	
	// [133] VAR1 ::= '?' VARNAME
	let VAR1 = seq2_ret1("?".lit().ws(), VARNAME).thene({|v| return(variable((v)))});
	
	// [98] Var ::= VAR1 | VAR2
	let Var = VAR1;
	
	// [96] VarOrTerm ::= Var | GraphTerm
	let VarOrTerm = Var.or(GraphTerm);
	
	// [95] GraphNode ::= VarOrTerm | TriplesNode
	let GraphNode = VarOrTerm;
	
	// [88] PathPrimary ::= IRIref | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
	let PathPrimary = IRIref.thene({|v| return(typed_literal(v, "http://www.w3.org/2001/XMLSchema#anyURI"))});
	
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
	let VerbSimple = Var;
	
	// [75] Object ::= GraphNode
	let Object = GraphNode;
	
	// [74] ObjectList ::= Object ( ',' Object )*
	let ObjectList = Object;
	
	// [80] VerbPath	::= Path
	let VerbPath = Path;
	
	// [78] PropertyListNotEmptyPath	::= (VerbPath | VerbSimple) ObjectList ( ';' ( ( VerbPath | VerbSimple ) ObjectList )? )*
	let PropertyListNotEmptyPath= seq2(VerbPath.or(VerbSimple), ObjectList)
		{|prop, object| result::ok([prop, object])};
		
	// [77] TriplesSameSubjectPath ::= VarOrTerm PropertyListNotEmptyPath | TriplesNode PropertyListPath 
	let TriplesSameSubjectPath = seq2(VarOrTerm, PropertyListNotEmptyPath)
		{|subject, e| result::ok({subject: subject, predicate: e[0], object: e[1]})};
		
	// [58] OptionalGraphPattern ::= 'OPTIONAL' GroupGraphPattern
	let GroupGraphPattern_ptr = @mut return(group([]));
	let GroupGraphPattern_ref = forward_ref(GroupGraphPattern_ptr);
	
	let OptionalGraphPattern = seq2("OPTIONAL".lit().ws(), GroupGraphPattern_ref)
		{|_o, a| result::ok(optional(@a))};
	
	// [57] GraphPatternNotTriples ::= GroupOrUnionGraphPattern | OptionalGraphPattern | MinusGraphPattern | GraphGraphPattern | ServiceGraphPattern | Filter | Bind
	let GraphPatternNotTriples = OptionalGraphPattern;
	
	// [56] TriplesBlock ::= TriplesSameSubjectPath ('.' TriplesBlock?)?
	let TriplesBlock = seq2(list(TriplesSameSubjectPath, ".".lit().ws()), ".".lit().ws().optional())
		{|patterns, _r|
			if vec::len(patterns) == 1u
			{
				result::ok(basic(patterns[0]))
			}
			else
			{
				result::ok(group(vec::map(patterns, {|p| @basic(p)})))
			}
		};
	
	// [55] GroupGraphPatternSub ::= TriplesBlock? (GraphPatternNotTriples '.'? TriplesBlock?)*		*****
	let ggps_suffix = seq3(GraphPatternNotTriples, ".".lit().ws().optional(), TriplesBlock.optional())
		{|gpnt, _d, tb|
			if option::is_some(tb)
			{
				result::ok(group([@gpnt, @(tb.get())]))
			}
			else
			{
				result::ok(gpnt)
			}
		};
	let GroupGraphPatternSub = seq2(TriplesBlock.optional(), ggps_suffix.r0())
		{|tb, gp|
			let patterns =
				if option::is_some(tb)
				{
					[tb.get()] + gp
				}
				else
				{
					gp
				};
			
			if vec::len(patterns) == 1u
			{
				result::ok(patterns[0])
			}
			else
			{
				result::ok(group(vec::map(patterns, {|p| @p})))
			}
		};
		
	// [54] GroupGraphPattern ::= '{' (SubSelect | GroupGraphPatternSub) '}'
	let GroupGraphPattern = seq3_ret1("{".lit().ws(), GroupGraphPatternSub, "}".lit().ws());
	*GroupGraphPattern_ptr = GroupGraphPattern;
	
	// [17] WhereClause ::= 'WHERE'? GroupGraphPattern
	let WhereClause = seq2_ret1(("WHERE".liti().ws()).optional(), GroupGraphPattern);
	
	// [9] SelectClause ::= 'SELECT' ('DISTINCT' | 'REDUCED')? ((Var | ('(' Expression 'AS' Var ')'))+ | '*')
	let select_suffix = or_v([
		Var.r1(),
		"*".lit().ws().thene({|_x| return([variable("*")])})]);
		
	let SelectClause = seq2_ret1("SELECT".liti().ws(), select_suffix);
		
	// [7] SelectQuery ::= SelectClause DatasetClause* WhereClause SolutionModifier
	let SelectQuery = seq2(SelectClause, WhereClause)
		{|patterns, algebra| result::ok((patterns, algebra))};
		
	// [6] PrefixDecl ::= 'PREFIX' PNAME_NS IRI_REF
	let PrefixDecl = seq3("PREFIX".lit().ws(), PNAME_NS.ws(), IRI_REF)
		{|_p, ns, ref| result::ok({prefix: str::slice(ns, 0u, str::len(ns)-1u), path: ref})};
	
	// [4] Prologue ::= (BaseDecl | PrefixDecl)*
	let Prologue = PrefixDecl.r0();
	
	// [2] Query ::= Prologue (SelectQuery | ConstructQuery | DescribeQuery | AskQuery) BindingsClause
	let Query = seq2(Prologue, SelectQuery)
		{|p, s| build_parser(p, tuple::first(s), tuple::second(s))};
	
	// [1] QueryUnit ::= Query
	let QueryUnit = Query;
	
	ret QueryUnit;
}

// namespaces are from the PREFIX clauses
// patterns are from the SELECT clause
// algebra is from the WHERE clause
fn build_parser(namespaces: [namespace], patterns: [pattern], algebra: algebra) -> result::result<selector, str>
{
	let variables = vec::filter(patterns) {|p| alt p {variable(_l) {true} _ {false}}};
	let names = vec::map(variables) {|p| alt p {variable(n) {n} _ {fail}}};
	
	let dupes = find_dupes(names);
	if vec::is_empty(dupes)
	{
		if vec::is_not_empty(namespaces)
		{
			result::ok(eval(names, expand(namespaces, algebra)))
		}
		else
		{
			result::ok(eval(names, algebra))
		}
	}
	else
	{
		result::err(#fmt["Select clause has duplicates: %s", str::connect(dupes, " ")])
	}
}
