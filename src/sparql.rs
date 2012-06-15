import rparse::*;
import query::*;

// We want to be able to compile a query into patterns and reuse the result
// with arbitrary stores or with stores that are changing. So we use this
// intermediate enum which (for efficiency) we convert into a pattern when
// the query is run against an actual store.
enum compiled_pattern
{
	variable_binding(str),
	string_literal(str, str),		// value + lang (which may be empty)
	iri_literal(str)
}

type compiled_triple_pattern = {subject: compiled_pattern, predicate: compiled_pattern, object: compiled_pattern};

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

//fn prefixed_name() -> parser<str>
//{
	// [163] PN_LOCAL_ESC ::=  '\' ( '_' | '~' | '.' | '-' | '!' | '$' | '&' | "'" | '(' | ')' | '*' | '+' | ',' | ';' | '=' | ':' | '/' | '?' | '#' | '@' | '%' )
	// [162] HEX ::= [0-9] | [A-F] | [a-f]
	// [161] PERCENT ::= '%' HEX HEX
	// [160] PLX ::= PERCENT | PN_LOCAL_ESC
	// [159] PN_LOCAL ::= (PN_CHARS_U | [0-9] | PLX ) ( ( PN_CHARS | '.' | PLX )* ( PN_CHARS | PLX ) ) ? >
	// [158] PN_PREFIX	::= PN_CHARS_BASE ((PN_CHARS|'.')* PN_CHARS)?
	// [157] PN_CHARS	::= PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
	// [155] PN_CHARS_U ::= PN_CHARS_BASE | '_'
	// [154] PN_CHARS_BASE	::= [A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
	// [131] PNAME_LN	::= PNAME_NS PN_LOCAL
	// [130] PNAME_NS	::= PN_PREFIX? ':'
	// [127] PrefixedName ::= PNAME_LN | PNAME_NS
//}

// http://www.w3.org/TR/sparql11-query/#grammar
fn make_parser() -> parser<selector>
{
	// [149] STRING_LITERAL_LONG2 ::= '"""' ( ( '"' | '""' )? ( [^"\] | ECHAR ) )* '"""'
	let STRING_LITERAL_LONG2 = seq3_ret1("\"\"\"".lit(), scan0(bind long_char('"', _, _)), "\"\"\"".lit().s0());
	
	// [148] STRING_LITERAL_LONG1 ::= "'''" ( ( "'" | "''" )? ( [^'\] | ECHAR ) )* "'''"
	let STRING_LITERAL_LONG1 = seq3_ret1("'''".lit(), scan0(bind long_char('\'', _, _)), "'''".lit().s0());
	
	// [147] STRING_LITERAL2 ::= '"' ( ([^"\\\n\r]) | ECHAR )* '"'
	let STRING_LITERAL2 = seq3_ret1("\"".lit(), scan0(bind short_char('"', _, _)), "\"".lit().s0());
	
	// [146] STRING_LITERAL1 ::= "'" ( ([^'\\\n\r]) | ECHAR )* "'"
	let STRING_LITERAL1 = seq3_ret1("'".lit(), scan0(bind short_char('\'', _, _)), "'".lit().s0());
	
	// [135] LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
	let LANGTAG = langtag();
	
	// [129] IRI_REF	 ::= '<' ([^<>"{}|^`\]-[#x00-#x20])* '>'
	let IRI_REF = seq3_ret1("<".lit(), match0(iri_char), ">".lit().s0());
	
	// [126] IRIref ::= IRI_REF | PrefixedName
	let IRIref = IRI_REF;
	
	// [125] String ::= STRING_LITERAL1 | STRING_LITERAL2 | STRING_LITERAL_LONG1 | STRING_LITERAL_LONG2
	let String = or_v([STRING_LITERAL_LONG1, STRING_LITERAL_LONG2, STRING_LITERAL1, STRING_LITERAL2]);
	
	// [119] RDFLiteral ::= String ( LANGTAG | ( '^^' IRIref ) )?
	let RDFLiteral1 = String.thene({|v| return(string_literal(v, ""))});
	
	let RDFLiteral2 = seq2(String, LANGTAG)
		{|v, l| result::ok(string_literal(v, l))};
	
	let RDFLiteral = or_v([RDFLiteral2, RDFLiteral1]);
	
	// [99] GraphTerm	::= IRIref | RDFLiteral | NumericLiteral |	BooleanLiteral |	BlankNode |	NIL
	let GraphTerm = or_v([
		RDFLiteral,
		IRIref.thene({|v| return(iri_literal(v))})
	]);
	
	// [156] VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*
	let VARNAME = identifier().s0();
	
	// [133] VAR1 ::= '?' VARNAME
	let VAR1 = seq2_ret1("?".lit().s0(), VARNAME).thene({|v| return(variable_binding((v)))});
	
	// [98] Var ::= VAR1 | VAR2
	let Var = VAR1;
	
	// [96] VarOrTerm ::= Var | GraphTerm
	let VarOrTerm = Var.or(GraphTerm);
	
	// [95] GraphNode ::= VarOrTerm | TriplesNode
	let GraphNode = VarOrTerm;
	
	// [88] PathPrimary ::= IRIref | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
	let PathPrimary = IRIref.thene({|v| return(iri_literal(v))});
	
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
		
	// [56] TriplesBlock ::= TriplesSameSubjectPath ( '.' TriplesBlock? )?
	let TriplesBlock = TriplesSameSubjectPath;
	
	// [55] GroupGraphPatternSub ::= TriplesBlock? ( GraphPatternNotTriples '.'? TriplesBlock? )*
	let GroupGraphPatternSub = TriplesBlock;
		
	// [54] GroupGraphPattern ::= '{' ( SubSelect | GroupGraphPatternSub ) '}'
	let GroupGraphPattern = seq3_ret1("{".lit().s0(), GroupGraphPatternSub, "}".lit().s0());
	
	// [17] WhereClause ::= 'WHERE'? GroupGraphPattern
	let WhereClause = seq2_ret1(("WHERE".liti().s0()).optional(), GroupGraphPattern);
	
	// [9] SelectClause ::= 'SELECT' ( 'DISTINCT' | 'REDUCED' )? ( ( Var | ( '(' Expression 'AS' Var ')' ) )+ | '*' )
	let SelectClause = seq2_ret1("SELECT".liti().s0(), Var.r1());
		
	// [7] SelectQuery ::= SelectClause DatasetClause* WhereClause SolutionModifier
	let SelectQuery = seq2(SelectClause, WhereClause)
		{|names, matchers|
			let variables = vec::filter(names) {|p| alt p {variable_binding(_l) {true} _ {false}}};
			let names = vec::map(variables) {|p| alt p {variable_binding(n) {n} _ {fail}}};
			
			let dupes = find_dupes(names);
			if vec::is_empty(dupes)
			{
				result::ok(select(names, matchers))
			}
			else
			{
				result::err(#fmt["Select clause has duplicates: %s", str::connect(dupes, " ")])
			}
		};
	
	// [2] Query ::= Prologue ( SelectQuery | ConstructQuery | DescribeQuery | AskQuery ) BindingsClause
	let Query = SelectQuery;
	
	// [1] QueryUnit ::= Query
	let QueryUnit = Query;
	
	ret QueryUnit;
}

