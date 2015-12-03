
use std::convert;
use std::io;

use std::fmt;
use std::result;


pub trait BasicWrite {
	
    fn write_str(&mut self, s: &str) ;

    fn write_char(&mut self, c: char) ;

}

// TODO: might have to remove this, if it's polluting namespace?
impl BasicWrite for String {
	
    fn write_str(&mut self, str: &str) {
    	self.push_str(str);
    }
	
    fn write_char(&mut self, ch: char) {
    	self.push(ch);
    }
	
}


/* -----------------  ----------------- */

pub type Result<T> = result::Result<T, CommonException2>;
pub type Void = Result<()>;

pub trait CommonException {
	
	fn writeMessage(&self, writer: &mut BasicWrite) ;
	
}

pub type CommonException2 = Box<CommonException>; 


fn writeSafeDisplay(displayObj : &fmt::Display, writer: &mut BasicWrite) {
	
	struct _BasicWrite<'a>(&'a mut BasicWrite);
	
	impl<'a> fmt::Write for _BasicWrite<'a> {
		fn write_str(&mut self, str: &str) -> fmt::Result {
	    	self.0.write_str(str);
	    	Ok(())
	    }
		
	    fn write_char(&mut self, ch: char) -> fmt::Result {
	    	self.0.write_char(ch);
	    	Ok(())
	    }
	    
	}
	
	fmt::write(&mut _BasicWrite(writer), format_args!("{}", displayObj))
		.expect("displayObj object should not result an error.");
	
}

impl convert::From<io::Error> for CommonException2 {
	fn from(obj: io::Error) -> Self {
		
		struct _CommonException(io::Error);
		impl CommonException for _CommonException {
			fn writeMessage(&self, writer: &mut BasicWrite) {
				writeSafeDisplay(&self.0, writer);
			}
		}
		
		Box::new(_CommonException(obj))
	}
}

impl convert::From<String> for CommonException2 {
	fn from(obj: String) -> Self {
		
		struct _CommonException(String);
		impl CommonException for _CommonException {
			fn writeMessage(&self, writer: &mut BasicWrite) {
				writer.write_str(&self.0);
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