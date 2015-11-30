
use std::io;

pub fn toStringToken(string : &str, out : &mut io::Write) -> io::Result<()> {
	
	use std::fmt::Write;
	
	for ch in string.chars() {
		let mut buf = String::new();
		buf.write_char(ch).unwrap();
		
		try!(out.write_all(buf.as_bytes()));
	
	}
	
	Ok(())
}