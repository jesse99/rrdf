// Hard-coded triple stores used for unit testing. These use a few different schemas:
// 1) http://www.w3.org/TR/vcard-rdf/
// Profile for electronix business cards.
// 2) https://github.com/edumbill/doap/wiki
// Profile used to describe open source software projects.

// Namespaces:
// got: 	<http://awoiaf.westeros.org/index.php/>
// v:	<http://www.w3.org/2006/vcard/ns#>

fn got_cast1() -> [triple]
{
	[
		{subject: iri("got:Eddard_Stark"), property: "v:fn", object: string("Eddard Stark")},
		{subject: iri("got:Eddard_Stark"), property: "v:nickname", object: string("Ned")}
	]
}

fn got_cast3() -> [triple]
{
	[
		{subject: iri("got:Eddard_Stark"), property: "v:fn", object: string("Eddard Stark")},
		{subject: iri("got:Eddard_Stark"), property: "v:nickname", object: string("Ned")},
		{subject: iri("got:Eddard_Stark"), property: "v:honorific-prefix", object: string("Lord")},
		{subject: iri("got:Eddard_Stark"), property: "v:org", object: reference(blank("_:o1"))},
		{subject: blank("_:o1"), property: "v:organisation-name", object: string("Small Council")},
		{subject: blank("_:o1"), property: "v:organisation-unit", object: string("Hand")},
		
		{subject: iri("got:Jon_Snow"), property: "v:fn", object: string("Jon Snow")},
		{subject: iri("got:Jon_Snow"), property: "v:nickname", object: string("Lord Snow")},
		{subject: iri("got:Jon_Snow"), property: "v:org", object: reference(blank("_:o1"))},
		{subject: blank("_:o1"), property: "v:organisation-name", object: string("Night's Watch")},
		{subject: blank("_:o1"), property: "v:organisation-unit", object: string("Stewards")},
		
		{subject: iri("got:Sandor_Clegane"), property: "v:fn", object: string("Sandor Clegane")},
		{subject: iri("got:Sandor_Clegane"), property: "v:nickname", object: string("The Hound")}
	]
}
