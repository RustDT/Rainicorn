extern crate rust_parse_describe;

use rust_parse_describe::*;

fn main() {
		
	/* -----------------  ----------------- */
	
	let source = r#"
	
use std::io; 

struct Foo {
	blah : u32, 
	xpto : &Str,
} 
fn func() { } 
trait Trait { 
	fn func(param : Type);
}

"#;
	
	parse_describe::parse_analysis(source);
	
	parse_describe::parse_analysis("fn foo(");
}
