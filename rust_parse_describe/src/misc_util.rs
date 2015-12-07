use std::io;
use std::fmt;

use std::io::Write;

pub struct StdoutWrite<'l>(pub &'l mut io::Stdout);

impl<'l> fmt::Write for StdoutWrite<'l> {
	
	fn write_str(&mut self, s: &str) -> fmt::Result {
		match self.0.write_all(s.as_bytes()) {
			Ok(_) => Ok(()),
			Err(_) => Err(fmt::Error),
		}
	}
	
}