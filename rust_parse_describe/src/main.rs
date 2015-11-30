extern crate rust_parse_describe;

use rust_parse_describe::*;

fn main() {
		
	/* -----------------  ----------------- */
	
	let source = "struct Foo { } fn func() {  } trait Trait { } ";
	
	parse_describe::parse_analisys(source);
	
	parse_describe::parse_analisys("fn ");
}
