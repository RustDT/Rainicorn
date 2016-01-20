# rust_parse_describe
rust_parse_describe is a tool intended for use by Rust IDEs/editors. 

It performs a single operation, a "parse-analysis" of a Rust source file (using lib_syntax, the rustc parser), and returns useful information such as:
 * Parse errors (if any). This can be used to provide on-the-fly parse errors reporting in the editor.
 * The structural elements of the source file (that is, the top-level definitions). This can be used to provide an editor outline, or provider the block source ranges for editor block folding.

The rust_parse_describe tool is a work-in-progress.

### API

*THIS IS PROVISIONAL UNTIL FIRST RELEASE*

Sample input:
```
use std::io; 

struct Foo {
	blah : u32, 
	xpto : &Str,
} 
fn func() { } 
trait Trait { 
	fn func(param : Type);
}
```
sample output:
```
RUST_PARSE_DESCRIBE 0.1 {
MESSAGES {
}
Use { "std::io" { 0:0 0:12 } }
Struct { "Foo" { 2:0 5:1 }
  Var { "blah" { 3:1 3:11 } }
  Var { "xpto" { 4:1 4:12 } }
}
Function { "func" { 6:0 6:13 } }
Trait { "Trait" { 7:0 9:1 }
  Function { "func" { 8:1 8:23 } }
}
}
```
--

Sample input:
```
fn foo(
  blah
```
sample output:
```
RUST_PARSE_DESCRIBE 0.1 {
MESSAGES { 
  { ERROR { 1:6 1:6 } "this file contains an un-closed delimiter" }
  { INFO { 0:6 0:7 } "did you mean to close this delimiter?" }
}
}
```

