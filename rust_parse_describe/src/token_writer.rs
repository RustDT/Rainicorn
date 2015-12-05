use ::core_util::*;

use std::result;

pub fn writeStringToken<ERR> (string : &str, out : &mut CharOutput<ERR>) -> result::Result<(), ERR> {
	
	use std::fmt::Write;
	
	for ch in string.chars() {
		
		try!(out.write_char(ch));
		
	}
	
	Ok(())
}

#[cfg(test)]
fn test_writeStringToken() {
	
	fn check_writeStringToken(string : &str, expected : &str) {
		let mut result = String::new();
		writeStringToken(string, &mut result as &mut CharOutput<()>);
		assert_eq!(result, expected);
	}
	
	check_writeStringToken("abc", r#""abc""#);
	
	check_writeStringToken("a\"bc\"", r#""a\"bc\"""#);
}