# rust_parse_describe
rust_parse_describe is a tool intended for use by Rust IDEs/editors. 

It performs a single operation, a "parse-analysis" of a Rust source file, and returns useful information such as:
 * Parse errors (if any). This can be used to provide on-the-fly parse errors reporting in the editor.
 * The structural elements of the source file (that is, the top-level definitions). This can be used to provide an editor outline, or the ranges for editor folding.
 
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
RUST_PARSE_DESCRIBE 0.1

Use { "std::io" { 3 0 3 12 } }
Struct { "Foo" { 5 0 8 1 }
  Var { "blah" { 6 1 6 11 } }
  Var { "xpto" { 7 1 7 12 } }
}
Function { "func" { 9 0 9 13 } }
Trait { "Trait" { 10 0 12 1 }
  Function { "func" { 11 1 11 23 } }
}
```
--

Sample input:
```
fn foo(
```
sample output:
```
RUST_PARSE_DESCRIBE 0.1
MESSAGE { help { 1 6 1 7 } "did you mean to close this delimiter?" }
MESSAGE { error { 1 7 1 7 } "this file contains an un-closed delimiter " }
```

