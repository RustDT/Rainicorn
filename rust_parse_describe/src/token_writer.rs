use ::util::core::*;

use std::result;
use std::fmt;

pub use std::cell::{ RefCell , RefMut };
pub use std::rc::{ Rc };

/// Write a parse structure into a serialized format
pub struct TokenWriter {
	pub out : Rc<RefCell<fmt::Write>>,
}

impl fmt::Debug for TokenWriter {
	
	fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("[TokenWriter]")
	}
	
}

impl TokenWriter {
	
	pub fn getCharOut(&self) -> RefMut<fmt::Write + 'static> {
		self.out.borrow_mut()
	}
	
	pub fn writeStringToken(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		self::writeStringToken(string, &mut* self.getCharOut())
	}
	
	pub fn writeRaw(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		self.getCharOut().write_str(string)
	}
	
	pub fn writeTextToken(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		self.getCharOut().write_str(string)
		//FIXME: check escapes
//		self::writeStringToken(string, &mut* self.out)
	}
	
}

pub fn writeStringToken<OUT : ?Sized + fmt::Write>(string : &str, out : &mut OUT) 
	-> fmt::Result 
//pub fn writeStringToken<ERR, OUT : ?Sized + CharOutput<ERR>>(string : &str, out : &mut OUT) 
//	-> result::Result<(), ERR> 
{
	
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
//	writeStringToken(string, &mut result as &mut CharOutput<()>).unwrap();
	writeStringToken(string, &mut result).unwrap();
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