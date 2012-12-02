//use io::WriterUtil;
//use test_data::*;
//use test_helpers::*;
use std::time;

fn big_store() -> Store
{
	let store = Store(~[Namespace {prefix: ~"wiki", path: ~"http://en.wikipedia.org/wiki/"}], @HashMap());
	
	let phylums = ~[~"Acanthocephala", ~"Acoelomorpha", ~"Annelida", ~"Arthropoda", ~"Brachiopoda", ~"Bryozoa", ~"Chaetognatha", ~"Chordata", ~"Cnidaria", ~"Ctenophora", ~"Cycliophora", ~"Echinodermata", ~"Entoprocta"];
	let classes = ~[~"Acoelomorpha", ~"Archiannelida", ~"Clitellata", ~"Myzostomida", ~"Polychaeta", ~"Arachnida", ~"Merostomata", ~"Pycnogonida", ~"Branchiopoda", ~"Cephalocarida"];
	let families = ~[~"Abyssocottidae", ~"Acanthuridae", ~"Acestrorhynchidae", ~"Achiridae", ~"Achiropsettidae", ~"Acipenseridae", ~"Acropomatidae", ~"Adrianichthyidae", ~"Ageneiosidae", ~"Agonidae"];
	
	let rng = core::rand::seeded_rng(&~[1u8]);
	for uint::range(0, 15_000) |_i|
	{
		if rng.gen_float() <= 0.1
		{
			let mut entries = ~[
				(~"wiki:episode", @IntValue(rng.gen_i64())),
				(~"wiki:name", @StringValue(rng.gen_str(10), ~"")),
			];
			if rng.gen_float() <= 0.1
			{
				entries.push((~"wiki:kenney_dies", @StringValue(rng.gen_str(8), ~"")));
			}
			store.add(rng.gen_str(16), entries);
		}
		else
		{
			store.add(rng.gen_str(16), ~[
				(~"wiki:phylum", @StringValue(rng.choose(phylums), ~"")),
				(~"wiki:class", @StringValue(rng.choose(classes), ~"")),
				(~"wiki:family", @StringValue(rng.choose(families), ~"")),
			]);
		}
	}
	store
}

// This is very much like a very slow query gnos run.
//
// Times				State
// 7.276 7.355 7.414	Before optimization work (but with -O which seems to speed queries up by 35%, note that ---opt-level 3 did not help)
// 0.887 0.801 0.913	Take advantage of new rust features, mostly more useable managed pointers and inherited mutability
#[test]
fn speed()
{
	let expr = ~"
	PREFIX
		wiki: <http://en.wikipedia.org/wiki/>
	SELECT
		?subject ?episode ?name ?dies
	WHERE
	{
		?subject wiki:episode ?episode .
		?subject wiki:name ?name .
		OPTIONAL
		{
			?subject wiki:kenny_dies ?dies .
		}
	}";
	
	let store = big_store();
	let compiled = compile(expr);
	let start = time::precise_time_s();
	match compiled
	{
		result::Ok(selector) =>
		{
			match selector(&store)
			{
				result::Ok(ref actual) =>
				{
					let elapsed = time::precise_time_s() - start;
					io::println(fmt!("%? rows returned in %.3f seconds", actual.rows.len(), elapsed));
					assert actual.rows.len() == 1545;		// should be valid as long as the rust RNG doesn't change
				}
				result::Err(ref mesg) =>
				{
					fail fmt!("Eval error: %s", *mesg);
				}
			}
		}
		result::Err(ref mesg) =>
		{
			 fail fmt!("Parse error: %s", *mesg);
		}
	}
}
