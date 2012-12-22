use tests::test_data::*;
use tests::test_helpers::*;

#[test]
fn solution_to_str()
{
	let expr = ~"
PREFIX got: <http://awoiaf.westeros.org/index.php/>
PREFIX v: <http://www.w3.org/2006/vcard/ns#>
SELECT 
	?s ?p ?o WHERE {?s ?p ?o}";
	let store = test_data::got_cast1();
	
	match compile(expr)
	{
		result::Ok(selector) =>
		{
			match selector(&store)
			{
				result::Ok(ref actual) =>
				{
					let actual = actual.sort();
					let actual = actual.to_str();
					let expected = ~"0 s: got:Eddard_Stark, p: v:fn, o: \"Eddard Stark\"\n1 s: got:Eddard_Stark, p: v:nickname, o: \"Ned\"\n";
					assert check_strs(actual, expected);
				}
				result::Err(ref mesg) =>
				{
					fail fmt!("Eval error: %s", *mesg);
				}
			}
		}
		result::Err(ref mesg) =>
		{
			fail fmt!("Parse error: %s", *mesg)
		}
	}
}
