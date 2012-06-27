import operands::*;
import test_helpers::*;

#[test]
fn invalid_values()
{
	let op = object_to_operand({value: "23xx", kind: "http://www.w3.org/2001/XMLSchema#integer", lang: ""});
	assert check_operands(op, invalid_value("'23xx' is not a valid integer."));
	
	let op = object_to_operand({value: "2..3", kind: "http://www.w3.org/2001/XMLSchema#double", lang: ""});
	assert check_operands(op, invalid_value("'2..3' is not a valid floating point number."));
	
	let op = object_to_operand({value: "05062012", kind: "http://www.w3.org/2001/XMLSchema#dateTime", lang: ""});
	assert check_operands(op, error_value("'05062012' is not an ISO 8601 dateTime."));
}
