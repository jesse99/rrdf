#[doc = "The types exported by the rrdf library. The most important of which are store, triple, solution, and selector."];

#[doc = "The function returned by compile and invoked to execute a SPARQL query.

Returns a solution or a 'runtime' error."]
type selector = fn@ (store) -> result::result<solution, str>;

