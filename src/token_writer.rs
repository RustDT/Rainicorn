// Copyright 2015 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
	
	pub fn writeRaw(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		self.getCharOut().write_str(string)
	}
	
	pub fn writeStringToken(&mut self, string : &str) -> result::Result<(), fmt::Error> {
		try!(write_escaped_string(string, &mut* self.getCharOut()));
		
		self.getCharOut().write_char(' ')
	}
	
	pub fn writeRawToken(&mut self, string : &str) -> Void {
		
		for ch in string.chars() {
			if ch.is_whitespace() 
				|| ch == '{' || ch == '}'  
				|| ch == '(' || ch == ')'
				|| ch == '[' || ch == ']'
			{
				return Err("Cannot write raw token".into());
			}
		}
		
		try!(self.getCharOut().write_str(string));
		try!(self.getCharOut().write_char(' '));
		
		Ok(())
	}
	
}

#[test]
fn test__writeRawToken() {
	
	fn writeRawToken_toString(string : &str) -> GResult<String> {
		let outRc : Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
		
		let result = TokenWriter { out: outRc.clone() }.writeRawToken(string);
		match result {
			Ok(_) => Ok(unwrap_Rc_RefCell(outRc)),
			Err(error) => Err(error),
		}
	}
	
	assert_eq!(writeRawToken_toString("blah").ok().unwrap(), r#"blah "#);
	writeRawToken_toString("bl ah").unwrap_err();
	writeRawToken_toString("bl{ah").unwrap_err();
}

/* ----------------- some parser/serialize utils ----------------- */

pub fn write_escaped_string<OUT : ?Sized + fmt::Write>(string : &str, out : &mut OUT) 
	-> fmt::Result 
//pub fn writeStringToken<ERR, OUT : ?Sized + CharOutput<ERR>>(string : &str, out : &mut OUT) 
//	-> result::Result<(), ERR> 
{
	
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


#[test]
fn test__write_escaped_string() {
	
	fn writeStringToken_toString(string : &str) -> String {
		let mut result = String::new();
		write_escaped_string(string, &mut result).unwrap();
		result
	}
	
	assert_eq!(writeStringToken_toString(""), r#""""#);
	assert_eq!(writeStringToken_toString("abc"), r#""abc""#);
	assert_eq!(writeStringToken_toString(r#"-"-"#), r#""-\"-""#);
	assert_eq!(writeStringToken_toString(r#"""#), r#""\"""#);
	assert_eq!(writeStringToken_toString(r#"\"#), r#""\\""#);
	assert_eq!(writeStringToken_toString(r#"--\"-"#), r#""--\\\"-""#);
	assert_eq!(writeStringToken_toString(r#"---\"#), r#""---\\""#);
}