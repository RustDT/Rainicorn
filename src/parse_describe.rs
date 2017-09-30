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

use source_model::*;

use syntex_syntax::syntax::ast;
use syntex_syntax::parse::{self, ParseSess};
use syntex_syntax::visit;
use syntex_syntax::codemap::{self, MultiSpan, CodeMap};
use syntex_errors::{Handler, Level, DiagnosticBuilder};
use syntex_errors::emitter;

use std::boxed::Box;
use std::path::Path;

use token_writer::TokenWriter;

use std::cell::RefCell;
use std::rc::*;
use std::io;
use std::io::Write;
use std::fmt;
use std::path::PathBuf;
use std::env;

pub fn parse_analysis_for_Stdout(source: &str) {
    parse_analysis(source, StdoutWrite(io::stdout())).ok();
    println!("");
    io::stdout().flush().ok();
}

pub fn parse_analysis<T: fmt::Write + 'static>(source: &str, out: T) -> GResult<T> {
    let (messages, elements) = parse_crate_with_messages(source);

    let outRc = Rc::new(RefCell::new(out));
    write_parse_analysis_do(messages, elements, outRc.clone())?;
    let res = unwrap_Rc_RefCell(outRc);
    return Ok(res);
}

use std::thread;
use std::sync::{Arc, Mutex};

pub fn parse_crate_with_messages(source: &str) -> (Vec<SourceMessage>, Vec<StructureElement>) {

    let messages = Arc::new(Mutex::new(vec![]));
    let elements = {
        let source = String::from(source);
        let messages = messages.clone();

        let worker_thread = thread::Builder::new()
            .name("parser_thread".to_string())
            .spawn(move || parse_crate_with_messages_do(&source, messages))
            .unwrap();

        worker_thread.join().unwrap_or(vec![])
    };

    let messages: Mutex<Vec<SourceMessage>> = Arc::try_unwrap(messages).ok().unwrap();
    let messages: Vec<SourceMessage> = messages.into_inner().unwrap();

    return (messages, elements);
}

pub fn parse_crate_with_messages_do(source: &str,
                                    messages: Arc<Mutex<Vec<SourceMessage>>>)
                                    -> Vec<StructureElement> {
    use structure_visitor::StructureVisitor;

    let mut elements = vec![];

    let fileLoader = Box::new(DummyFileLoader::new());
    let codemap = Rc::new(CodeMap::with_file_loader(fileLoader));

    let krate = parse_crate(source, codemap.clone(), messages.clone());

    if let Some(krate) = krate {
        let mut visitor: StructureVisitor = StructureVisitor::new(&codemap);
        visit::walk_crate(&mut visitor, &krate);

        elements = visitor.elements;
    }
    return elements;
}

/* -----------------  ----------------- */


use std::ffi::OsStr;

/// A FileLoader that loads any file successfully
pub struct DummyFileLoader {
    modName: &'static OsStr,
}

impl DummyFileLoader {
    fn new() -> DummyFileLoader {
        DummyFileLoader { modName: OsStr::new("mod.rs") }
    }
}

impl codemap::FileLoader for DummyFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        return path.file_name() == Some(self.modName);
    }

    fn abs_path(&self, path: &Path) -> Option<PathBuf> {
        if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            env::current_dir().ok().map(|cwd| cwd.join(path))
        }
    }

    fn read_file(&self, _path: &Path) -> io::Result<String> {
        Ok(String::new())
    }
}


struct MessagesHandler {
    codemap: Rc<CodeMap>,
    messages: Arc<Mutex<Vec<SourceMessage>>>,
}


fn parse_crate<'a>(source: &str,
                   codemap: Rc<CodeMap>,
                   messages: Arc<Mutex<Vec<SourceMessage>>>)
                   -> Option<ast::Crate> {
    let emitter = MessagesHandler::new(codemap.clone(), messages.clone());

    let handler = Handler::with_emitter(true, false, Box::new(emitter));
    let sess = ParseSess::with_span_handler(handler, codemap.clone());

    let krate_result = parse_crate_do(source, &sess);

    return match krate_result {
        Ok(_krate) => Some(_krate),
        Err(mut db) => {
            db.emit();
            None
        }
    };
}

pub fn parse_crate_do<'a>(source: &str, sess: &'a ParseSess) -> parse::PResult<'a, ast::Crate> {
    let source = source.to_string();

    let name = "_file_module_".to_string();

    //    We inlined: let mut parser = parse::new_parser_from_source_str(&sess, cfg, name, source);

    let filemap = sess.codemap().new_filemap(name, None, source);

    // filemap_to_tts but without a panic
    let tts = {
        let srdr = parse::lexer::StringReader::new(&sess.span_diagnostic, filemap);
        let mut p1 = parse::parser::Parser::new(sess, Box::new(srdr));

        p1.parse_all_token_trees()?
    };

    let trdr = parse::lexer::new_tt_reader(&sess.span_diagnostic, None, tts);
    let mut parser = parse::parser::Parser::new(sess, Box::new(trdr));

    return parser.parse_crate_mod();
}




impl MessagesHandler {
    fn new(codemap: Rc<CodeMap>, messages: Arc<Mutex<Vec<SourceMessage>>>) -> MessagesHandler {
        MessagesHandler {
            codemap: codemap,
            messages: messages,
        }
    }

    fn write_message_handled(&mut self,
                             sourcerange: Option<SourceRange>,
                             msg: &str,
                             severity: Severity) {

        let msg = SourceMessage {
            severity: severity,
            sourcerange: sourcerange,
            message: String::from(msg),
        };

        let mut messages = self.messages.lock().unwrap();
        messages.push(msg);
    }
}

impl emitter::Emitter for MessagesHandler {
    fn emit(&mut self, db: &DiagnosticBuilder) {
        let msg: &str = &db.message;
        let code: Option<&String> = db.code.as_ref();
        let lvl: Level = db.level;

        let multispan: &MultiSpan = &db.span;

        if let Some(code) = code {
            io::stderr()
                .write_fmt(format_args!("Code: {}\n", code))
                .unwrap();
            panic!("What is code: Option<&str>??");
        }

        let sourceranges: Vec<_> = multispan
            .primary_spans()
            .iter()
            .map(|span| -> SourceRange { SourceRange::new(&self.codemap, *span) })
            .collect();

        for sourcerange in sourceranges {
            self.write_message_handled(Some(sourcerange), msg, level_to_status_level(lvl));
        }
    }
}

fn level_to_status_level(lvl: Level) -> Severity {
    match lvl { 
        Level::PhaseFatal => panic!("Level::PhaseFatal"),
        Level::Bug => panic!("Level::BUG"), 
        Level::Cancelled => panic!("Level::CANCELLED"),
        Level::Help | Level::Note => Severity::INFO, 
        Level::Warning => Severity::WARNING,
        Level::Error | Level::Fatal => Severity::ERROR,
    }
}

impl MessagesHandler {}


/* ----------------- describe writting ----------------- */

pub fn write_parse_analysis_do(messages: Vec<SourceMessage>,
                               elements: Vec<StructureElement>,
                               out: Rc<RefCell<fmt::Write>>)
                               -> Void {

    let mut tokenWriter = TokenWriter { out: out };

    tokenWriter.write_raw("RUST_PARSE_DESCRIBE 1.0 {\n")?;
    write_parse_analysis_contents(messages, elements, &mut tokenWriter)?;
    tokenWriter.write_raw("\n}")?;

    Ok(())
}

pub fn write_parse_analysis_contents(messages: Vec<SourceMessage>,
                                     elements: Vec<StructureElement>,
                                     tokenWriter: &mut TokenWriter)
                                     -> Void {

    tokenWriter.write_raw("MESSAGES {\n")?;
    for msg in messages {
        output_message(tokenWriter, msg.sourcerange, &msg.message, &msg.severity)?;
    }
    tokenWriter.write_raw("}\n")?;


    for element in elements {
        write_structure_element(tokenWriter, &element, 0)?;
    }

    Ok(())
}

fn output_message(tokenWriter: &mut TokenWriter,
                  opt_sr: Option<SourceRange>,
                  msg: &str,
                  lvl: &Severity)
                  -> Void {

    tokenWriter.write_raw("{ ")?;

    output_Level(&lvl, tokenWriter)?;

    output_opt_SourceRange(&opt_sr, tokenWriter)?;

    tokenWriter.write_string_token(msg)?;

    tokenWriter.write_raw("}\n")?;

    Ok(())
}


pub fn output_Level(lvl: &Severity, writer: &mut TokenWriter) -> Void {

    writer.write_raw_token(lvl.to_string())?;

    Ok(())
}

pub fn output_SourceRange(sr: &SourceRange, tw: &mut TokenWriter) -> Void {
    tw.write_raw("{ ")?;
    {
        let mut out = tw.get_output();
        out.write_fmt(format_args!("{}:{} {}:{} ", 
            sr.start_pos.line-1, sr.start_pos.col.0,
            sr.end_pos.line-1, sr.end_pos.col.0,
        ))?;
    }
    tw.write_raw("}")?;

    Ok(())
}

pub fn output_opt_SourceRange(sr: &Option<SourceRange>, writer: &mut TokenWriter) -> Void {

    match sr {
        &None => writer.write_raw("{ }")?,
        &Some(ref sr) => output_SourceRange(sr, writer)?,
    }

    writer.write_raw(" ")?;

    Ok(())
}


pub fn write_indent(tokenWriter: &mut TokenWriter, level: u32) -> Void {
    writeNTimes(&mut *tokenWriter.get_output(), ' ', level * 2)?;
    Ok(())
}

pub fn write_structure_element(tw: &mut TokenWriter,
                               element: &StructureElement,
                               level: u32)
                               -> Void {
    tw.write_raw_token(element.kind.to_string())?;

    tw.write_raw("{ ")?;

    tw.write_string_token(&element.name)?;

    output_SourceRange(&element.sourcerange, tw)?;

    tw.get_output().write_str(" {}")?; // name source range, Not Supported

    tw.get_output().write_str(" ")?;
    tw.write_string_token(&element.type_desc)?;

    tw.get_output().write_str("{}")?; // attribs, Not Supported

    if element.children.is_empty() {
        tw.get_output().write_str(" ")?;
    } else {
        let level = level + 1;

        for child in &element.children {
            tw.get_output().write_str("\n")?;
            write_indent(tw, level)?;
            write_structure_element(tw, child, level)?;
        }

        tw.get_output().write_str("\n")?;
        write_indent(tw, level - 1)?;
    }

    tw.get_output().write_str("}")?;

    Ok(())
}


#[cfg(test)]
mod parse_describe_tests {

    use parse_describe::*;
    use source_model::*;
    use token_writer::TokenWriter;
    use util::core::*;
    use util::tests::check_equal;
    use util;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn test_write_structure_element(name: &str,
                                    kind: StructureElementKind,
                                    sr: SourceRange,
                                    type_desc: String,
                                    expected: &str) {
        let stringRc = Rc::new(RefCell::new(String::new()));
        {
            let name = String::from(name);
            let element = StructureElement {
                name: name,
                kind: kind,
                sourcerange: sr,
                type_desc: type_desc,
                children: vec![],
            };
            let mut tw = TokenWriter { out: stringRc.clone() };

            write_structure_element(&mut tw, &element, 0).ok();
        }

        assert_eq!(unwrap_Rc_RefCell(stringRc).trim(), expected);
    }

    #[test]
    fn write_structure_element__tests() {
        test_write_structure_element("blah",
                                     StructureElementKind::Var,
                                     source_range(1, 0, 2, 5),
                                     "desc".to_string(),
                                     r#"Var { "blah" { 0:0 1:5 } {} "desc" {} }"#);
    }


    #[test]
    fn parse_analysis__tests() {
        test_parse_analysis("", "");

        test_parse_analysis(" #blah ",
                            r#"{ ERROR { 0:2 0:6 } "expected `[`, found `blah`" }"#);

        test_parse_analysis(
            "fn foo(\n  blah",
            r#"
{ ERROR { 1:6 1:6 } "this file contains an un-closed delimiter" }
{ ERROR { 1:6 1:6 } "expected one of `:` or `@`, found `)`" }
{ ERROR { 1:6 1:6 } "expected one of `->`, `where`, or `{`, found `<eof>`" }
"#,
        );

        // Test a lexer panic
        test_parse_analysis(
            "const a = '",
            r#"{ ERROR { 0:10 0:11 } "character literal may only contain one codepoint: '" }"#,
        );

        // test `?` syntax shorthand for try:
        test_parse_analysis("fn foo() { 123? }",
                            &("}\n".to_string() + r#"Function { "foo" { 0:0 0:17 } {} "()" {}"#));
    }

    fn test_parse_analysis(source: &str, expected_msgs: &str) {
        let result = parse_analysis(source, String::new()).ok().unwrap();
        let mut result: &str = &result;

        result = assert_surrounding_string("RUST_PARSE_DESCRIBE 1.0 {", result, "}");

        result = assert_starts_with("MESSAGES {", result.trim());
        expected_msgs.replace("\r\n", "\n");
        result = assert_starts_with(expected_msgs.trim(), result.trim());
        check_equal(result.trim(), "}");
    }

    fn assert_surrounding_string<'a>(start: &str, string: &'a str, end: &str) -> &'a str {
        let mut string: &str = string;

        string = assert_starts_with(start, string);
        string = assert_ends_with(string, end);

        return string;
    }

    fn assert_starts_with<'a>(start: &str, string: &'a str) -> &'a str {
        util::tests::assert_starts_with(string, start);
        return &string[start.len()..];
    }

    fn assert_ends_with<'a>(string: &'a str, end: &str) -> &'a str {
        assert!(string.ends_with(end),
                "`{}` does not end with `{}`",
                string,
                end);
        return &string[0..string.len() - end.len()];
    }

}
