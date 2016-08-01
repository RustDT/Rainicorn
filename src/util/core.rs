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

use std::convert;
use std::io;

use std::fmt;
use std::result;


pub type CommonException = Box<CommonExceptionT>;
pub type Result<T> = result::Result<T, CommonException>;
pub type Void = Result<()>;


pub trait CharOutput<ERR> {
	
    fn write_str(&mut self, s: &str) -> result::Result<(), ERR>;
	
    fn write_char(&mut self, c: char) -> result::Result<(), ERR>;
	
}


pub trait BasicCharOutput {
	
    fn put_str(&mut self, s: &str) ;

    fn put_char(&mut self, c: char) ;

}

//impl fmt::Write for BasicCharOutput {
//	fn write_str(&mut self, str: &str) -> fmt::Result {
//    	self.put_str(str);
//    	Ok(())
//    }
//	
//    fn write_char(&mut self, ch: char) -> fmt::Result {
//    	self.put_char(ch);
//    	Ok(())
//    }
//}


impl<ERR> CharOutput<ERR> for BasicCharOutput {
	
    fn write_str(&mut self, s: &str) -> result::Result<(), ERR> {
    	BasicCharOutput::put_str(self, s);
    	Ok(())
    }
	
    fn write_char(&mut self, c: char) -> result::Result<(), ERR> {
    	BasicCharOutput::put_char(self, c);
    	Ok(())
    }
	
}

//// TODO: might have to remove this, if it's polluting namespace?
impl BasicCharOutput for String {
	
    fn put_str(&mut self, str: &str) {
    	self.push_str(str);
    }
	
    fn put_char(&mut self, ch: char) {
    	self.push(ch);
    }
	
}

impl<ERR> CharOutput<ERR> for String {
	
	fn write_str(&mut self, s: &str) -> result::Result<(), ERR> {
		self.put_str(s);
		Ok(())
    }
	
    fn write_char(&mut self, c: char) -> result::Result<(), ERR> {
    	self.put_char(c);
    	Ok(())
    }
}

/* ----------------- CommonCharOutput ----------------- */

pub type CommonCharOutput = CharOutput<CommonException>;

/* -----------------  ----------------- */

pub trait CommonExceptionT {
	
	fn write_message(&self, writer: &mut CommonCharOutput) -> Void;
	
	fn write_message_to_string(&self) -> String {
		let mut str = String::new();
		self.write_message(&mut str).ok(); // Should not error because it's writting to a string
		str
	}
	
}

impl fmt::Display for CommonExceptionT {
	
	fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result {
		let to_string = self.write_message_to_string();
		fmt.write_str(&to_string)
	}
	
}

impl fmt::Debug for CommonExceptionT {
	
	fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result {
		<Self as fmt::Display>::fmt(self, fmt)
	}
	
}


pub struct StringCommonException(String);

impl StringCommonException {
	
	pub fn create(string : String) -> Box<StringCommonException> {
		Box::new(StringCommonException(string))
	}
	
}

impl CommonExceptionT for StringCommonException {
	fn write_message(&self, out: &mut CommonCharOutput) -> Void {
		out.write_str(&self.0)
	}
}

struct FmtDisplayCommonException<T : fmt::Display>(T);

impl<T : fmt::Display> CommonExceptionT for FmtDisplayCommonException<T> {
	fn write_message(& self, out: &mut CommonCharOutput) -> Void {
		write_display_to_char_out(& self.0, out)
	}
}

fn write_display_to_char_out(display: & fmt::Display, out: &mut CommonCharOutput) -> Void {
	let string = format!("{}", display);
	out.write_str(&string)
}

/*
fn write_display_to_BasicCharOut(display : &fmt::Display, out: &mut BasicCharOutput) {
	
	struct _BasicWrite<'a>(&'a mut BasicCharOutput);
	
	impl<'a> fmt::Write for _BasicWrite<'a> {
		fn write_str(&mut self, str: &str) -> fmt::Result {
	    	self.0.put_str(str);
	    	Ok(())
	    }
		
	    fn write_char(&mut self, ch: char) -> fmt::Result {
	    	self.0.put_char(ch);
	    	Ok(())
	    }
	    
	}
	
	fmt::write(&mut _BasicWrite(out), format_args!("{}", display))
		.expect("displayObj object should not result an error.");
	
}

#[test]
fn test_write_display_to_BasicCharOut() {
	
	struct _Display(());
	impl fmt::Display for _Display {
		fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "Blah {}", "XXX")
		}
	}
	
	let mut result = String::new();
	write_display_to_BasicCharOut(&_Display(()), &mut result);
	assert_eq!(result, "Blah XXX");
}
*/

/* ----------------- convert to CommonException ----------------- */


impl convert::From<io::Error> for CommonException {
	fn from(obj: io::Error) -> Self {
		Box::new(FmtDisplayCommonException(obj))
	}
}

impl convert::From<fmt::Error> for CommonException {
	fn from(obj: fmt::Error) -> Self {
		Box::new(FmtDisplayCommonException(obj))
	}
}

impl convert::From<String> for CommonException {
	fn from(obj: String) -> Self {
		Box::new(StringCommonException(obj))
	}
}

use std::num;

impl convert::From<num::ParseIntError> for CommonException {
	fn from(obj: num::ParseIntError) -> Self {
		Box::new(FmtDisplayCommonException(obj))
	}
}


#[test]
fn test_convert() {
	
	fn test() -> Void {
		try!(Err(String::from("ERROR")));
		Ok(())
	}
	
	test().unwrap_err();
}



/* -----------------  ----------------- */

use std::rc::Rc;
use std::cell::RefCell;

pub fn unwrap_Rc_RefCell<T>(this: Rc<RefCell<T>>) -> T {
	let ures : result::Result<RefCell<_>, _> = Rc::try_unwrap(this);
	match ures {
		Ok(refCell) => return refCell.into_inner(),
		Err(_) => panic!("std::Rc unwrap failed")
	}
}