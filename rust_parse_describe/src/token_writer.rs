use ::core_util::*;
use ::misc_util::*;

use std::result;
use std::fmt;


/// Write a parse structure into a serialized format
pub struct TokenWriter {
	pub out : Box<StdoutWrite>, // TODO: change to ARC or something
	//out : &'a mut StdoutWrite,
}

impl<'a> TokenWriter {
	
	pub fn writeStringToken(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		self::writeStringToken(string, &mut* self.out)
	}
	
}

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