# Rainicorn
Rainicorn is a tool intended for use by Rust IDEs. 

It currenly performs a single operation, a "parse-analysis" of a Rust source file (using lib_syntax, the rustc parser), and returns useful information such as:
 * Parse errors (if any). This can be used to provide on-the-fly parse errors reporting in the editor.
 * The structural elements of the source file (that is, the top-level definitions). This can be used to provide an editor outline, or provider the block source ranges for editor block folding.

##### Installation
Run `cargo install --git https://github.com/RustDT/Rainicorn --tag version_1.x`

##### Changelog:
 * 1.3 - Support for unions, `?` syntax shortcut for `try`, `pub extern crate` (was warning a before).
   * Unfortunately, impl names are no longer displayed, see: https://github.com/serde-rs/syntex/issues/106.

##### Future TODO:
Note that parse-describe functionality should eventually be subsumed by [Language Server Protocol](https://github.com/Microsoft/language-server-protocol) functionality, namely:
 * "PublishDiagnostics Notification" to provide parse errors.
 * "Document Symbols Request" to provide document structural elements (the tree can be recreated from the flat symbols using the range information, see https://github.com/Microsoft/language-server-protocol/issues/112)

### parse_describe API (1.0)

Run the parse_describe tool, provide the Rust source code into stdin. Output supplied to stdout. All operation output is in the fornat of a simple block tokens language (described below). 

Example input (Rust source code):
```
fn foo(
  blah
```
Example output:
```
RUST_PARSE_DESCRIBE 1.0 {
MESSAGES { 
  { ERROR { 1:6 1:6 } "this file contains an un-closed delimiter" }
  { INFO { 0:6 0:7 } "did you mean to close this delimiter?" }
}
}
```
--
Example input:
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
Example output:
```
RUST_PARSE_DESCRIBE 1.0 {
MESSAGES {
}
Use { "std::io" { 0:0 0:12 } {} {} {} }
Struct { "Foo" { 2:0 5:1 } {} {} {}
  Var { "blah" { 3:1 3:11 } {} {} {} }
  Var { "xpto" { 4:1 4:12 } {} {} {} }
}
Function { "func" { 6:0 6:13 } {} {} {} }
Trait { "Trait" { 7:0 9:1 } {} {} {}
  Function { "func" { 8:1 8:23 } {} {} {} }
}
}
```
--
#### Spec:

* OUTPUT = `RUST_PARSE_DESCRIBE version=TEXT {`  `{` MESSAGE* `}`  SOURCE_ELEMENT* `}`
* MESSAGE = `{` severity=SEVERITY source_range=SOURCE_RANGE text=QUOTED_STRING `}`
* SEVERITY = `ERROR` | `WARNING` | `INFO`
* SOURCE_RANGE = `{` start_pos=POSITION end_pos=POSITION `}`
* POSITION = QUOTED_STRING 
  * A string value in the format `line:column` or `@absolute_offset`. line, column and offset are zero-based indexes. Example `0:2`, `"5:10"` or `@250`.
* SOURCE_ELEMENT = ELEMENT_KIND `{` name=QUOTED_STRING source_range=SOURCE_RANGE name_source_range=SOURCE_RANGE TYPE_DESC ATTRIBUTES `}`
* ELEMENT_KIND 
  * One of: Var, Function, Struct, Impl, Trait, Enum, EnumVariant, ExternCrate, Mod, Use, TypeAlias;
* TYPE_DESC = QUOTED_STRING 
  * A string value with a description of the "type" of the given element. Currently this will contain the signature of functions, or the type of Const/Static elements.
* ATTRIBUTES = `{}` 
  * **No info currently supplied, but saved for future usage**

#### Block tokens:
This data language only has 3 types of tokens:
* *WHITESPACE*: Ignored. There are no comments (yet).
* *TEXT*: Either raw text (ie `Foo` or a quoted string (`"Foo"` or `"blah \" blah "`). #TODO spec
* *BRACE*: An open or closing brace, either one of: `{`, `}`, `(`, `)`, `[`, `]`.

The only structural requirement of this language is that the braces be correctly balanced.
