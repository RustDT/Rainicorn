
use std::convert;
use std::io;

use std::fmt;
use std::result;

pub trait CharOutput<ERR> {
	
    fn write_str(&mut self, s: &str) -> result::Result<(), ERR>;
	
    fn write_char(&mut self, c: char) -> result::Result<(), ERR>;
	
}


pub trait BasicCharOutput {
	
    fn put_str(&mut self, s: &str) ;

    fn put_char(&mut self, c: char) ;

}

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

/* -----------------  ----------------- */

pub trait CommonException {
	
	fn writeMessage(&self, writer: &mut BasicCharOutput) ;
	
}

pub type BCommonException = Box<CommonException>;
pub type Result<T> = result::Result<T, BCommonException>;
pub type Void = Result<()>;


impl fmt::Display for CommonException {
	
	fn fmt(&self, fmt : &mut fmt::Formatter) -> fmt::Result {
		
		// FIXME: optimize this by write directly to fmt through an adapter, dont crate intermediate string
		
		let mut str = String::new();
		self.writeMessage(&mut str);
		fmt.write_str(&str)
		
//		struct _BasicWrite<'a>(&'a mut fmt::Formatter<'a>);
//		impl<'a> BasicWrite for _BasicWrite<'a> {
//			fn write_str(&mut self, str: &str) {
//				self.0.write_str(str);
//		    }
//			
//		    fn write_char(&mut self, ch: char) {
//		    	self.0.write_char(ch);
//		    }
//		}
//		
//		Ok(())
	}
	
}



struct FmtDisplayCommonException<T : fmt::Display>(T);

impl<T : fmt::Display> CommonException for FmtDisplayCommonException<T> {
	fn writeMessage(&self, writer: &mut BasicCharOutput) {
		writeSafeDisplay(&self.0, writer);
	}
}


fn writeSafeDisplay(displayObj : &fmt::Display, out: &mut BasicCharOutput) {
	
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
	
	fmt::write(&mut _BasicWrite(out), format_args!("{}", displayObj))
		.expect("displayObj object should not result an error.");
	
}

impl convert::From<io::Error> for BCommonException {
	fn from(obj: io::Error) -> Self {
		Box::new(FmtDisplayCommonException(obj))
	}
}

impl convert::From<fmt::Error> for BCommonException {
	fn from(obj: fmt::Error) -> Self {
		Box::new(FmtDisplayCommonException(obj))
	}
}

impl convert::From<String> for BCommonException {
	fn from(obj: String) -> Self {
		
		struct _CommonException(String);
		impl CommonException for _CommonException {
			fn writeMessage(&self, out: &mut BasicCharOutput) {
				out.put_str(&self.0);
			}
		}
		
		Box::new(_CommonException(obj))
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

#[test]
fn test_fmt_error() {
	
	struct _Display(());
	impl fmt::Display for _Display {
		fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "Blah {}", "XXX")
		}
	}
	
	let mut result = String::new();
	writeSafeDisplay(&_Display(()), &mut result);
	assert_eq!(result, "Blah XXX");
}


/* -----------------  ----------------- */

use std::rc::Rc;
use std::cell::RefCell;

pub fn unwrapRcRefCell<T>(this: Rc<RefCell<T>>) -> T {
	let ures : result::Result<RefCell<_>, _> = Rc::try_unwrap(this);
	match ures {
		Ok(refCell) => return refCell.into_inner(),
		Err(_) => panic!("std::Rc unwrap failed")
	}
}