use ::core_util::*;

use std::io;
use std::fmt;

use std::io::Write;

pub struct StdoutWrite(pub io::Stdout);

impl fmt::Write for StdoutWrite {
	
	fn write_str(&mut self, s: &str) -> fmt::Result {
		match self.0.write_all(s.as_bytes()) {
			Ok(_) => Ok(()),
			Err(_) => Err(fmt::Error),
		}
	}
	
}

impl CharOutput<fmt::Error> for StdoutWrite {
	
    fn write_str(&mut self, string: &str) -> fmt::Result {
    	fmt::Write::write_str(self, string)
    }
	
    fn write_char(&mut self, c: char) -> fmt::Result {
    	fmt::Write::write_char(self, c)
    }
	
}