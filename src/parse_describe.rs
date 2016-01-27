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

use ::util::core::*;
use ::util::string::*;
use ::source_model::*;

use ::syntex_syntax::syntax::ast;
use ::syntex_syntax::parse::{ self, ParseSess };
use ::syntex_syntax::visit;
use ::syntex_syntax::codemap:: { self, Span, CodeMap};
use ::syntex_syntax::errors:: { Handler, RenderSpan, Level, emitter };

use std::boxed::Box;
use std::path::Path;

use ::token_writer::TokenWriter;

use std::cell::RefCell;
use std::rc::*;
use std::io;
use std::io::Write;
use std::fmt;

/* -----------------  ----------------- */

pub fn parse_analysis_forStdout(source : &str) {
	parse_analysis(source, StdoutWrite(io::stdout())).ok();
	println!("");
	io::stdout().flush().ok();
}

pub fn parse_analysis<T : fmt::Write + 'static>(source : &str, out : T) -> Result<T> {
	let (messages, elements) = parse_crate_with_messages(source);
	 
	let outRc = Rc::new(RefCell::new(out));
	try!(write_parse_analysis_do(messages, elements, outRc.clone()));
	let res = unwrap_Rc_RefCell(outRc);
	return Ok(res);
}


pub fn parse_crate_with_messages(source: &str) -> (Vec<SourceMessage>, Vec<StructureElement>) {
	use ::structure_visitor::StructureVisitor;

	let fileLoader = Box::new(DummyFileLoader::new());
	let codemap = Rc::new(CodeMap::with_file_loader(fileLoader));
	
	let messages = Rc::new(RefCell::new(vec![]));
	let krate = parse_crate(source, codemap.clone(), messages.clone());

	let mut elements = vec![];
	if let Some(krate) = krate {
		let mut visitor : StructureVisitor = StructureVisitor::new(&codemap);  
		visit::walk_crate(&mut visitor, &krate);
		
		elements = visitor.elements;
	}
	
	let messages = Rc::try_unwrap(messages).ok().unwrap().into_inner();
	return (messages, elements);
}

/* -----------------  ----------------- */


use std::ffi::OsStr;

/// A FileLoader that loads any file successfully
pub struct DummyFileLoader {
   	modName : &'static OsStr,
}

impl DummyFileLoader {
	fn new() -> DummyFileLoader {
		DummyFileLoader { modName : OsStr::new("mod.rs") } 
	}
}

impl codemap::FileLoader for DummyFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
    	return path.file_name() == Some(self.modName);
    }
	
    fn read_file(&self, _path: &Path) -> io::Result<String> {
        Ok(String::new())
    }
}


struct MessagesHandler {
	codemap : Rc<CodeMap>,
	messages : Rc<RefCell<Vec<SourceMessage>>>,
}

 // FIXME: need to review this
unsafe impl ::std::marker::Send for MessagesHandler { }

fn parse_crate<'a>(source: &str, codemap: Rc<CodeMap>, messages: Rc<RefCell<Vec<SourceMessage>>>) -> Option<ast::Crate> 
{
	let emitter = MessagesHandler::new(codemap.clone(), messages.clone());
	
	let handler = Handler::with_emitter(true, false, Box::new(emitter));
	let sess = ParseSess::with_span_handler(handler, codemap.clone());
	
	let krate_result = parse_crate_do(source, &sess);
	
	return match krate_result {
		Ok(_krate) => { 
			Some(_krate) 
		}
		Err(mut db) => { 
			db.emit();
			None
		}
	}
}

pub fn parse_crate_do<'a>(source : &str, sess : &'a ParseSess) -> parse::PResult<'a, ast::Crate> 
{
	let source = source.to_string();
	
	let cfg = vec![];
	let name = "_file_module_".to_string();
	
//	We inlined: let mut parser = parse::new_parser_from_source_str(&sess, cfg, name, source); 

	let filemap = sess.codemap().new_filemap(name, source);
	
	// filemap_to_tts but without a panic
	let tts =
	{
	    let cfg = Vec::new();
	    let srdr = parse::lexer::StringReader::new(&sess.span_diagnostic, filemap);
	    let mut p1 = parse::parser::Parser::new(sess, cfg, Box::new(srdr));
	    
	    try!(p1.parse_all_token_trees())
	};
	
    let trdr = parse::lexer::new_tt_reader(&sess.span_diagnostic, None, None, tts);
    let mut parser = parse::parser::Parser::new(sess, cfg, Box::new(trdr));
	
	return parser.parse_crate_mod();
}




impl MessagesHandler {
	
	fn new(codemap: Rc<CodeMap>, messages: Rc<RefCell<Vec<SourceMessage>>>) -> MessagesHandler {
		MessagesHandler { codemap : codemap, messages : messages }
	}
	
	fn writeMessage_handled(&mut self, sourcerange : Option<SourceRange>, msg: &str, severity: Severity) {
		
		let msg = SourceMessage{ severity : severity , sourcerange : sourcerange,  message : String::from(msg) };
		
//		let mut messages = self.messages.lock().unwrap();
//		messages.push(msg);
		self.messages.borrow_mut().push(msg);
	}
	
}

impl emitter::Emitter for MessagesHandler {
	
    fn emit(&mut self, span: Option<Span>, msg: &str, code: Option<&str>, lvl: Level) {
    	
    	if let Some(code) = code {
   			io::stderr().write_fmt(format_args!("Code: {}\n", code)).unwrap();
   			panic!("What is code: Option<&str>??");
    	}
    	
		let sourcerange = match span {
			Some(span) => Some(SourceRange::new(&self.codemap, span)),
			None => None,
		};
		
		self.writeMessage_handled(sourcerange, msg, level_to_status_level(lvl));
    }
    
    fn custom_emit(&mut self, _: RenderSpan, msg: &str, lvl: Level) {
    	match lvl { Level::Help | Level::Note => return, _ => () }
    	
    	self.writeMessage_handled(None, msg, level_to_status_level(lvl));
    }
	
}

fn level_to_status_level(lvl: Level) -> Severity {
	match lvl { 
		Level::Bug => panic!("Level::BUG"), 
		Level::Cancelled => panic!("Level::CANCELLED"),
		Level::Help | Level::Note => Severity::INFO, 
		Level::Warning => Severity::WARNING,
		Level::Error | Level::Fatal => Severity::ERROR,
	}
}

impl MessagesHandler {
}


/* ----------------- describe writting ----------------- */

pub fn write_parse_analysis_do(messages: Vec<SourceMessage>, elements: Vec<StructureElement>, 
	out : Rc<RefCell<fmt::Write>>) -> Void {
	
	let mut tokenWriter = TokenWriter { out : out };
	
	try!(tokenWriter.writeRaw("RUST_PARSE_DESCRIBE 0.1 {\n"));
	try!(write_parse_analysis_contents(messages, elements, &mut tokenWriter));
	try!(tokenWriter.writeRaw("\n}"));
	
	Ok(())
}

pub fn write_parse_analysis_contents(messages: Vec<SourceMessage>, elements: Vec<StructureElement>, 
	tokenWriter : &mut TokenWriter) -> Void {
	
	try!(tokenWriter.writeRaw("MESSAGES {\n"));
	for msg in messages {
		try!(output_message(tokenWriter, msg.sourcerange, &msg.message, &msg.severity));
	}
	try!(tokenWriter.writeRaw("}\n"));
	
	
	for element in elements {
		try!(write_structure_element(tokenWriter, &element, 0));
	}
	
	Ok(())
}

fn output_message(tokenWriter: &mut TokenWriter, opt_sr : Option<SourceRange>, msg: & str, lvl: &Severity) 
	-> Void
{
	
	try!(tokenWriter.writeRaw("{ "));
	
	try!(outputString_Level(&lvl, tokenWriter));
	
	try!(outputString_optSourceRange(&opt_sr, tokenWriter));
	
	try!(tokenWriter.writeStringToken(msg));
	
	try!(tokenWriter.writeRaw("}\n"));
	
	Ok(())
}


pub fn outputString_Level(lvl : &Severity, writer : &mut TokenWriter) -> Void {
	
	try!(writer.writeRawToken(lvl.to_string()));
	
	Ok(())
}

pub fn outputString_SourceRange(sr : &SourceRange, tw : &mut TokenWriter) -> Void {
	try!(tw.writeRaw("{ "));
	{
		let mut out = tw.getCharOut(); 
		try!(out.write_fmt(format_args!("{}:{} {}:{} ", 
			sr.start_pos.line-1, sr.start_pos.col.0,
			sr.end_pos.line-1, sr.end_pos.col.0,
		)));
	}
	try!(tw.writeRaw("}"));
	
	Ok(())
}

pub fn outputString_optSourceRange(sr : &Option<SourceRange>, writer : &mut TokenWriter) -> Void {
	
	match sr {
		&None => try!(writer.writeRaw("{ }")) ,
		&Some(ref sr) => try!(outputString_SourceRange(sr, writer)) ,
	}
	
	try!(writer.writeRaw(" "));
	
	Ok(())
}


pub fn write_indent(tokenWriter : &mut TokenWriter, level : u32) -> Void {
	try!(writeNTimes(&mut *tokenWriter.getCharOut(), ' ', level * 2));
	Ok(())
}

pub fn write_structure_element(tw : &mut TokenWriter, element: &StructureElement, level: u32) -> Void
{
	try!(tw.writeRawToken(element.kind.to_String()));
	
	try!(tw.writeRaw("{ "));
	
	try!(tw.writeStringToken(&element.name));
	
	try!(outputString_SourceRange(&element.sourcerange, tw));
	
	try!(tw.getCharOut().write_str(" {}")); // name source range
	
	try!(tw.getCharOut().write_str(" {}")); // protection
	try!(tw.getCharOut().write_str(" {}")); // attribs
	
	
	if element.children.is_empty() {
		try!(tw.getCharOut().write_str(" "));
	} else {
		let level = level + 1;
		
		for child in &element.children {
			try!(tw.getCharOut().write_str("\n"));
			try!(write_indent(tw, level));
			try!(write_structure_element(tw, child, level));
		}
		
		try!(tw.getCharOut().write_str("\n"));
		try!(write_indent(tw, level-1));
	}
	
	try!(tw.getCharOut().write_str("}"));
	
	Ok(())
}

#[test]
fn tests_write_structure_element() {
	
	use ::std::rc::Rc;
	use ::std::cell::RefCell;
	
	fn test_writeStructureElement(name : &str, kind : StructureElementKind, sr: SourceRange, expected : &str) {
		let stringRc = Rc::new(RefCell::new(String::new()));
		{
			let name = String::from(name);
			let element = StructureElement { name: name, kind: kind, sourcerange: sr, children: vec![]}; 
			let mut tw = TokenWriter { out : stringRc.clone() };
			
			write_structure_element(&mut tw, &element, 0).ok();
		}
		
		assert_eq!(unwrap_Rc_RefCell(stringRc).trim(), expected);
	}
	
	test_writeStructureElement("blah", StructureElementKind::Var, sourceRange(1, 0, 2, 5), 
		r#"Var { "blah" { 0:0 1:5 } {} {} {} }"#);
}