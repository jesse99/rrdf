import rparse::*;
import rparse::misc::*;
import rparse::types::*;
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

// http://www.w3.org/TR/sparql11-query/#grammar
fn make_parser() -> parser<selector>
{
	// [156] VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*
	let VARNAME = identifier().space0();
	
	// [147] STRING_LITERAL2 ::= '"' ( ([^#x22#x5C#xA#xD]) | ECHAR )* '"'
	let STRING_LITERAL2 = sequence3(literal("\""), match_string_body('"'), literal("\""))
		{|_l, s, _r| result::ok(string_literal(s))};
	
	// [146] STRING_LITERAL1 ::= "'" ( ([^#x27#x5C#xA#xD]) | ECHAR )* "'"
	let STRING_LITERAL1 = sequence3(literal("'"), match_string_body('\''), literal("'"))
		{|_l, s, _r| result::ok(string_literal(s))};
	
	// [133] VAR1 ::= '?' VARNAME
	let VAR1 = sequence2(literal("?").space0(), VARNAME)
		{|_l, name| result::ok(variable(name))};
		
	// [125] String ::= STRING_LITERAL1 | STRING_LITERAL2 | STRING_LITERAL_LONG1 | STRING_LITERAL_LONG2
	let String = STRING_LITERAL1.or(STRING_LITERAL2);
		
	// [119] RDFLiteral ::= String ( LANGTAG | ( '^^' IRIref ) )?
	let RDFLiteral = String;
		
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
	let PropertyListNotEmptyPath= sequence2(VerbSimple, ObjectList)
		{|prop, object| result::ok([match_property(prop), match_object(object)])};
		
	// [77] TriplesSameSubjectPath ::= VarOrTerm PropertyListNotEmptyPath | TriplesNode PropertyListPath 
	let TriplesSameSubjectPath = sequence2(VarOrTerm, PropertyListNotEmptyPath)
		{|subject, plist| result::ok([match_subject(subject), plist[0], plist[1]])};
		
	// [56] TriplesBlock ::= TriplesSameSubjectPath ( '.' TriplesBlock? )?
	let TriplesBlock = TriplesSameSubjectPath;
	
	// [55] GroupGraphPatternSub ::= TriplesBlock? ( GraphPatternNotTriples '.'? TriplesBlock? )*
	let GroupGraphPatternSub = TriplesBlock;
		
	// [54] GroupGraphPattern ::= '{' ( SubSelect | GroupGraphPatternSub ) '}'
	let GroupGraphPattern = sequence3(literal("{").space0(), GroupGraphPatternSub, literal("}").space0())
		{|_l, matchers, _r| result::ok(matchers)};
	
	// [17] WhereClause ::= 'WHERE'? GroupGraphPattern
	let WhereClause = sequence2((literali("WHERE").space0()).optional("_"), GroupGraphPattern)
		{|_l, matchers| result::ok(matchers)};
	
	// [9] SelectClause ::= 'SELECT' ( 'DISTINCT' | 'REDUCED' )? ( ( Var | ( '(' Expression 'AS' Var ')' ) )+ | '*' )
	let SelectClause = sequence2(literali("SELECT").space0(), Var.repeat1("Variable's"))
		{|_l, names| result::ok(names)};
		
	// [7] SelectQuery ::= SelectClause DatasetClause* WhereClause SolutionModifier
	let SelectQuery = sequence2(SelectClause, WhereClause)
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

#[doc = "Return either a function capable of matching triples or a parse error.

Expr can be a subset of http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"SPARQL\"."]
fn compile(expr: str) -> result::result<selector, str>
{
	let parser = make_parser();
	result::chain_err(parse(parser, "sparql", expr))
	{|err|
		result::err(#fmt["%s on line %? col %?", err.mesg, err.line, err.col])
	}
}
