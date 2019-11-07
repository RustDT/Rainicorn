// Copyright 2015 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::util::core::*;

use std::fmt;
use std::result;

pub use std::cell::{RefCell, RefMut};
pub use std::rc::Rc;

/// Write a parse structure into a serialized format
pub struct TokenWriter {
    pub out: Rc<RefCell<dyn fmt::Write>>,
}

impl fmt::Debug for TokenWriter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("[TokenWriter]")
    }
}

impl TokenWriter {
    pub fn get_output(&self) -> RefMut<dyn fmt::Write + 'static> {
        self.out.borrow_mut()
    }

    pub fn write_raw(&mut self, string: &str) -> result::Result<(), fmt::Error> {
        self.get_output().write_str(string)
    }

    pub fn write_string_token(&mut self, string: &str) -> result::Result<(), fmt::Error> {
        write_escaped_string(string, &mut *self.get_output())?;

        self.get_output().write_char(' ')
    }

    pub fn write_raw_token(&mut self, string: &str) -> Void {
        for ch in string.chars() {
            if ch.is_whitespace() || ch == '{' || ch == '}' || ch == '(' || ch == ')' || ch == '[' || ch == ']' {
                return Err("Cannot write raw token".into());
            }
        }

        self.get_output().write_str(string)?;
        self.get_output().write_char(' ')?;

        Ok(())
    }
}

#[test]
fn test__write_raw_token() {
    fn write_raw_token_toString(string: &str) -> GResult<String> {
        let outRc: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let result = TokenWriter { out: outRc.clone() }.write_raw_token(string);
        match result {
            Ok(_) => Ok(unwrap_Rc_RefCell(outRc)),
            Err(error) => Err(error),
        }
    }

    assert_eq!(write_raw_token_toString("blah").ok().unwrap(), r#"blah "#);
    write_raw_token_toString("bl ah").unwrap_err();
    write_raw_token_toString("bl{ah").unwrap_err();
}

/* ----------------- some parser/serialize utils ----------------- */

pub fn write_escaped_string<OUT: ?Sized + fmt::Write>(string: &str, out: &mut OUT) -> fmt::Result
//pub fn write_string_token<ERR, OUT : ?Sized + CharOutput<ERR>>(string : &str, out : &mut OUT) 
//    -> result::Result<(), ERR>
{
    out.write_char('"')?;

    for ch in string.chars() {
        if ch == '"' || ch == '\\' {
            out.write_char('\\')?;
        }
        out.write_char(ch)?;
    }

    out.write_char('"')?;

    Ok(())
}

#[test]
fn test__write_escaped_string() {
    fn write_string_token_toString(string: &str) -> String {
        let mut result = String::new();
        write_escaped_string(string, &mut result).unwrap();
        result
    }

    assert_eq!(write_string_token_toString(""), r#""""#);
    assert_eq!(write_string_token_toString("abc"), r#""abc""#);
    assert_eq!(write_string_token_toString(r#"-"-"#), r#""-\"-""#);
    assert_eq!(write_string_token_toString(r#"""#), r#""\"""#);
    assert_eq!(write_string_token_toString(r#"\"#), r#""\\""#);
    assert_eq!(write_string_token_toString(r#"--\"-"#), r#""--\\\"-""#);
    assert_eq!(write_string_token_toString(r#"---\"#), r#""---\\""#);
}
