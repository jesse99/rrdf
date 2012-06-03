import to_str::to_str;

// ---- Types -----------------------------------------------------------------
#[doc = "An internationalized URI."]
type iri = str;

type duration = {negative: bool, years: uint, months: uint, days: uint, hours: uint, minutes: uint, seconds: f64};

#[doc = "Subject of a triple.

* iri is an internationalized absolute URI. Note that rdf does not interpret or decompose the IRIs.
* blank is a unique string used to identify subjects which do not have meaningful names."]
enum subject
{
	iri(iri),
	blank(str)
}

#[doc = "Object of a triple.

* reference identifies a subject.
* typed_literal is an arbitrary lexical value along with an IRI for its type.
* plain_literal is a string along with a language tag (e.g. \"en-us\") See http://tools.ietf.org/html/bcp47 \"Tags for Identifying Languages\".
* seq is sequentially ordered list of objects, possibly containing duplicates.
* bag is an un-ordered list of objects, possibly containing duplicates.
* alt is a list of alternative values (e.g. of download servers).
* xml is embedded xml, see: http://www.w3.org/TR/2011/WD-rdf11-concepts-20110830/#dfn-rdf-xmlliteral.
* The rest are primitive values based on the http://www.w3.org/TR/2001/REC-xmlschema-2-20010502/#built-in-datatypes \"XML Schema\".
"]
enum object
{
	reference(subject),
	typed_literal(str, iri),
	plain_literal(str, str),
	seq([object]),
	bag([object]),
	alt_([object]),
	xml(str),
	
	string(str),				// character strings
	boolean(bool),			// binary values
	
	decimal(str),				// arbitrary precision floating-point number, TODO: use a better type value
	integer(str),				// arbitrary precision integral number, TODO: use a better type value
	nonPositiveInteger(str),	// integer that cannot have negative values
	nonNegativeInteger(str),// integer that cannot have positive values
	long(i64),					// 64-bit integer
	int(i32),					// 32-bit integer
	short(i16),				// 16-bit integer
	byte(i8),					// 8-bit integer
	unsignedLong(u64),		// unsigned 64-bit integer
	unsignedInt(u32),		// unsigned 32-bit integer
	unsignedShort(u16),	// unsigned 16-bit integer
	unsignedByte(u8),		// unsigned 8-bit integer
	
	float(f32),					// 32-bit floating point number
	double(f64),				// 64-bit floating point number
	
	duration(duration),		// time span
	dateTime(str),			// specific instant in time, TODO: use a better type value
	time(str),					// instance in time that recurs every day, TODO: use a better type value
	date(str),					// a calendar date, TODO: use a better type value
	
	hexBinary([u8]),			// hex-encoded binary data
	base64Binary([u8]),		// Base64-encoded binary data
	
	anyURI(str),				// absolute or relative URI
	language(str)				// natural language identifier, see: http://xml.coverpages.org/iso639a.html
	// TODO: add gYearMonth, gYear, gMonthDay, gDay, gMonth, QName, NOTATION,
	// normalizedString, token, NMTOKEN, NMTOKENS, NCNAME, ID, IDREF, ENTITY, ENTITIES
}

#[doc = "A relationship between a subject and an object.

* subject identifies a resource, usually via an IRI.
* property is an IRI describing the relationship. Also known as a predicate.
* object is a IRI, literal, or blank node containing the value for the relationship.

Here is a psuedo-code example:

('https://github.com/jesse99/rrdf', 'http://purl.org/dc/terms/creator', 'Jesse Jones')
"]
type triple = {subject: subject, property: iri, object: object};

// ---- Exported Functions ----------------------------------------------------
impl of to_str for subject
{
	fn to_str() -> str
	{
		alt self
		{
			iri(v)			{"http://" + v}
			blank(v)		{v}
		}
	}
}

impl of to_str for object
{
	fn to_str() -> str
	{
		alt self
		{
			reference(v)			{v.to_str()}
			typed_literal(v, t)	{#fmt["(%s, %s)", v, t.to_str()]}
			plain_literal(v, t)		{#fmt["(%s, %s)", v, t]}
			seq(v)					{"[" + str::connect(vec::map(v, {|o| o.to_str()}), ", ") + "]"}
			bag(v)					{"{" + str::connect(vec::map(v, {|o| o.to_str()}), ", ") + "}"}
			alt_(v)					{"[" + str::connect(vec::map(v, {|o| o.to_str()}), " | ") + "]"}
			
			xml(v)			{v}
			string(v)		{#fmt["\"%s\"", v]}
			boolean(v)	{if (v) {"true"} else {"false"}}
			
			decimal(v)					{v}
			integer(v)						{v}
			nonPositiveInteger(v)		{v}
			nonNegativeInteger(v)		{v}
			long(v)						{#fmt["%?", v]}
			int(v)							{#fmt["%?", v]}
			short(v)						{#fmt["%?", v]}
			byte(v)						{#fmt["%?", v]}
			unsignedLong(v)				{#fmt["%?", v]}
			unsignedInt(v)				{#fmt["%?", v]}
			unsignedShort(v)			{#fmt["%?", v]}
			unsignedByte(v)				{#fmt["%?", v]}
			
			float(v)		{#fmt["%?", v]}
			double(v)		{#fmt["%?", v]}
			
			duration(v)	{#fmt["%?", v]}
			dateTime(v)	{#fmt["%?", v]}
			time(v)		{#fmt["%?", v]}
			date(v)		{#fmt["%?", v]}
			
			hexBinary(v)		{"0x" + str::connect(vec::map(v, {|e| #fmt["%02X", e as uint]}), "")}
			base64Binary(v)	{"0x" + str::connect(vec::map(v, {|e| #fmt["%02X", e as uint]}), "")}
			
			anyURI(v)			{"http://" + v}
			language(v)		{v}
		}
	}
}

impl of to_str for triple
{
	fn to_str() -> str
	{
		#fmt["{%s, %s, %s}", self.subject.to_str(), self.property.to_str(), self.object.to_str()]
	}
}

#[doc = "Returns an anonymous subject for the specified graph."]
fn get_blank(graph: [triple]) -> subject
{
	blank(#fmt["_:%?", vec::len(graph)])
}

// TODO: make this a const once rustc supports it
fn zero_duration() -> duration
{
	{negative: false, years: 0u, months: 0u, days: 0u, hours: 0u, minutes: 0u, seconds: 0f64}
}

fn seconds_to_duration(secs: f64) -> object
{
	if secs < 0.0
	{
		duration({negative: true, seconds: f64::abs(secs) with zero_duration()})
	}
	else
	{
		duration({seconds: secs with zero_duration()})
	}
}
