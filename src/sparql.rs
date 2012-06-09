import rparse::*;
import query::*;

export compile;

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

fn match_string_body_char(input: [char], i: uint, quote: char) -> uint
{ 
	let ch = input[i] as char;
	
	if ch == '\\' && option::is_some(str::find_char("tbnrf\"'", input[i+1u] as char))	// input ends with EOT so we don't need a range check here
	{
		// [150] ECHAR ::= '\' [tbnrf"']
		2u
	}
	else if ch != quote && ch != '\\' && ch != '\r' && ch != '\n' && ch != EOT
	{
		// [^#x22#x5C#xA#xD] or [^#x27#x5C#xA#xD]
		1u
	}
	else
	{
		0u
	}
}

fn match_string_body(quote: char) -> parser<str>
{
	{|input: state|
		let mut i = input.index;
		loop
		{
			let delta = match_string_body_char(input.text, i, quote);
			if delta > 0u
			{
				i += delta;
			}
			else
			{
				break;
			}
		}
		
		let text = str::from_chars(vec::slice(input.text, input.index, i));
		log_ok("match_string_body", input, {new_state: {index: i with input}, value: text})
	}
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

// http://www.w3.org/TR/sparql11-query/#grammar
fn make_parser() -> parser<selector>
{
	// [156] VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*
	let VARNAME = identifier().s0();
	
	// [147] STRING_LITERAL2 ::= '"' ( ([^#x22#x5C#xA#xD]) | ECHAR )* '"'
	let STRING_LITERAL2 = seq3_ret1("\"".lit(), match_string_body('"'), "\"".lit());
	
	// [146] STRING_LITERAL1 ::= "'" ( ([^#x27#x5C#xA#xD]) | ECHAR )* "'"
	let STRING_LITERAL1 = seq3_ret1("'".lit(), match_string_body('\''), "'".lit());
		
	// [135] LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
	let LANGTAG = langtag();
	
	// [133] VAR1 ::= '?' VARNAME
	let VAR1 = seq2_ret1("?".lit().s0(), VARNAME).thene({|v| return(variable((v)))});
	
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
	
	// [129] IRI_REF	 ::= '<' ([^<>"{}|^`\]-[#x00-#x20])* '>'
	// [126] IRIref ::= IRI_REF | PrefixedName
		
	// [125] String ::= STRING_LITERAL1 | STRING_LITERAL2 | STRING_LITERAL_LONG1 | STRING_LITERAL_LONG2
	let String = STRING_LITERAL1.or(STRING_LITERAL2);
	
	// [119] RDFLiteral ::= String ( LANGTAG | ( '^^' IRIref ) )?
	let RDFLiteral1 = String.thene({|v| return(constant(string(v)))});
	
	let RDFLiteral2 = seq2(String, LANGTAG)
		{|l, r| result::ok(constant(plain_literal(l, r)))};
	
	let RDFLiteral = RDFLiteral2.or(RDFLiteral1);
		
	// [99] GraphTerm	::= IRIref | RDFLiteral | NumericLiteral |	BooleanLiteral |	BlankNode |	NIL
	let GraphTerm = RDFLiteral;
	
	// [98] Var ::= VAR1 | VAR2
	let Var = VAR1;
	
	// [96] VarOrTerm ::= Var | GraphTerm
	let VarOrTerm = Var.or(GraphTerm);
	
	// [95] GraphNode ::= VarOrTerm | TriplesNode
	let GraphNode = VarOrTerm;
	
	// [81] VerbSimple ::= Var
	let VerbSimple = Var;
	
	// [75] Object ::= GraphNode
	let Object = GraphNode;
	
	// [74] ObjectList ::= Object ( ',' Object )*
	let ObjectList = Object;
	
	// [78] PropertyListNotEmptyPath	::= (VerbPath | VerbSimple) ObjectList ( ';' ( ( VerbPath | VerbSimple ) ObjectList )? )*
	let PropertyListNotEmptyPath= seq2(VerbSimple, ObjectList)
		{|prop, object| result::ok([match_property(prop), match_object(object)])};
		
	// [77] TriplesSameSubjectPath ::= VarOrTerm PropertyListNotEmptyPath | TriplesNode PropertyListPath 
	let TriplesSameSubjectPath = seq2(VarOrTerm, PropertyListNotEmptyPath)
		{|subject, plist| result::ok([match_subject(subject), plist[0], plist[1]])};
		
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
			let variables = vec::filter(names) {|p| alt p {variable(_l) {true} _ {false}}};
			let names = vec::map(variables) {|p| alt p {variable(n) {n} _ {fail}}};
			
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

#[doc = "Returns either a function capable of matching triples or a parse error.

Expr can be a subset of http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"SPARQL\"."]
fn compile(expr: str) -> result::result<selector, str>
{
	let parser = make_parser();
	result::chain_err(parse(parser, "sparql", expr))
	{|err|
		result::err(#fmt["%s on line %? col %?", err.mesg, err.line, err.col])
	}
}
