extern crate rainicorn_lib; 

use rainicorn_lib::parse_describe::*;


#[test]
fn parse_analysis_tests() {
	test_parse_analysis("", "");
	
	test_parse_analysis(" #blah ", r#"{ ERROR { 0:2 0:6 } "expected `[`, found `blah`" }"#);
	
	test_parse_analysis("fn foo(\n  blah", r#"
{ ERROR { 1:6 1:6 } "this file contains an un-closed delimiter" }
{ INFO { 0:6 0:7 } "did you mean to close this delimiter?" }"#);
	
	/* FIXME: lexer panics*/
//	test_parse_analysis("const a = '", r#""#);
}

fn test_parse_analysis(source : &str, expected_msgs : &str) {
	let result = parse_analysis(source, String::new()).ok().unwrap();
	let mut result : &str = &result;
	
	result = assert_surrounding_string("RUST_PARSE_DESCRIBE 0.1 {", result, "}");
	
	result = assert_starts_with("MESSAGES {", result.trim());
	result = assert_starts_with(expected_msgs.trim(), result.trim());
	result = assert_starts_with("}", result.trim());
	
	assert_eq!(result, "");
}

fn assert_surrounding_string<'a> (start : &str, string : &'a str, end : &str) -> &'a str {
	let mut string : &str = string;
	
	string = assert_starts_with(start, string);
	string = assert_ends_with(string, end);
	
	return string;
}

fn assert_starts_with<'a> (start : &str, string : &'a str) -> &'a str {
	assert!(string.starts_with(start), "`{}` does not start with `{}`", string, start);
	return &string[start.len() .. ];
}

fn assert_ends_with<'a> (string : &'a str, end : &str) -> &'a str {
	assert!(string.ends_with(end), "`{}` does not end with `{}`", string, end);
	return &string[0 .. string.len() - end.len()];
}

