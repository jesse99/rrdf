//import operands::*;
import test_helpers::*;

#[test]
fn invalid_values()
{
	let op = literal_to_object("23xx", "http://www.w3.org/2001/XMLSchema#integer", "");
	assert check_operands(op, invalid_value("23xx", "http://www.w3.org/2001/XMLSchema#integer"));
	
	let op = literal_to_object("2..3", "http://www.w3.org/2001/XMLSchema#double", "");
	assert check_operands(op, invalid_value("2..3", "http://www.w3.org/2001/XMLSchema#double"));
	
	let op = literal_to_object("05062012", "http://www.w3.org/2001/XMLSchema#dateTime", "");
	assert check_operands(op, error_value("'05062012' is not an ISO 8601 dateTime."));
}
