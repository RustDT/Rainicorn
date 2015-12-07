use ::core_util::*;

use std::result;

pub fn writeStringToken<ERR>(string : &str, out : &mut CharOutput<ERR>) -> result::Result<(), ERR> {
	
	use std::fmt::Write;
	
	try!(out.write_char('"'));
	
	for ch in string.chars() {
		
		if ch == '"' || ch == '\\' {
			try!(out.write_char('\\'));
		}
		try!(out.write_char(ch));
		
	}
	
	try!(out.write_char('"'));
	
	Ok(())
}

pub fn writeStringToken_toString(string : &str) -> String {
	let mut result = String::new();
	writeStringToken(string, &mut result as &mut CharOutput<()>).unwrap();
	result
}

#[test]
fn test_writeStringToken() {
	
	assert_eq!(writeStringToken_toString(""), r#""""#);
	assert_eq!(writeStringToken_toString("abc"), r#""abc""#);
	assert_eq!(writeStringToken_toString(r#"-"-"#), r#""-\"-""#);
	assert_eq!(writeStringToken_toString(r#"""#), r#""\"""#);
	assert_eq!(writeStringToken_toString(r#"\"#), r#""\\""#);
	assert_eq!(writeStringToken_toString(r#"--\"-"#), r#""--\\\"-""#);
	assert_eq!(writeStringToken_toString(r#"---\"#), r#""---\\""#);
	
}