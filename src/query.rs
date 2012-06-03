type result = {names: [str], rows: [[option<object>]]};	// len(names) == len(rows[x])

type binding = {name: str, value: option<object>};
type match = core::either::either<[binding], bool>;						// match succeeded if bindings or true
type matcher = fn@ (triple) -> match;

fn match(_triple: triple, _subject: option<object>, _property: option<object>, _object: option<object>) -> match
{
	core::either::right(false)
}

fn match_str_start(_triple: triple, _subject: option<object>, _property: option<object>, _object: option<object>) -> match
{
	core::either::right(false)
}

fn match_str_end(_triple: triple, _subject: option<object>, _property: option<object>, _object: option<object>) -> match
{
	core::either::right(false)
}

fn match_str_contains(_triple: triple, _subject: option<object>, _property: option<object>, _object: option<object>) -> match
{
	core::either::right(false)
}

// Each matching triple is associated with zero or more bindings.
fn match_triples(_triples: [triple], _matcher: matcher) -> ([[binding]], [triple])
{
	([], [])
}

// Returns triples where each triple satisfies every matcher.
fn query_all(_triples: [triple], _matchers: [matcher]) -> result
{
	{names: [""], rows: []}
}

