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

/* ----------------- Model ----------------- */

pub enum StructureElementKind {
	Var,
	Function,
	Struct,
	Impl,
	Trait,
	Enum,
	EnumVariant,
	ExternCrate,
	Mod,
	Use,
	TypeAlias,
}


use std::fmt;

impl StructureElementKind {
	pub fn writeString(&self, out : &mut fmt::Write) -> fmt::Result {
		match *self {
			StructureElementKind::Var => out.write_str("Var"),
			StructureElementKind::Function => out.write_str("Function"),
			StructureElementKind::Struct => out.write_str("Struct"),
			StructureElementKind::Impl => out.write_str("Impl"),
			StructureElementKind::Trait => out.write_str("Trait"),
			StructureElementKind::Enum => out.write_str("Enum"),
			StructureElementKind::EnumVariant => out.write_str("EnumVariant"),
			StructureElementKind::ExternCrate => out.write_str("ExternCrate"),
			StructureElementKind::Mod => out.write_str("Mod"),
			StructureElementKind::Use => out.write_str("Use"),
			StructureElementKind::TypeAlias => out.write_str("TypeAlias"),
		}
	}
}


/* -----------------  ----------------- */

pub fn parse_analysis_forStdout(source : &str) {
	parse_analysis(source, StdoutWrite(io::stdout())).ok();
	println!("");
	io::stdout().flush().ok();
}


use ::structure_visitor::StructureVisitor;

pub fn parse_analysis<T : fmt::Write + 'static>(source : &str, out : T) -> Result<T> {
	let outRc = Rc::new(RefCell::new(out));
	try!(parse_analysis_do(source, outRc.clone()));
	let res = unwrapRcRefCell(outRc);
	return Ok(res);
}

pub fn parse_analysis_do(source : &str, out : Rc<RefCell<fmt::Write>>) -> Void {
	
	let tokenWriter = TokenWriter { out : out };
	let tokenWriterRc : Rc<RefCell<TokenWriter>> = Rc::new(RefCell::new(tokenWriter));
	
	try!(tokenWriterRc.borrow_mut().writeRaw("RUST_PARSE_DESCRIBE 0.1 {\n"));
	try!(parse_analysis_contents(source, tokenWriterRc.clone()));
	try!(tokenWriterRc.borrow_mut().writeRaw("\n}"));
	
	Ok(())
}

pub fn parse_analysis_contents(source : &str, tokenWriterRc : Rc<RefCell<TokenWriter>>) -> Void {
	
	let fileLoader = Box::new(DummyFileLoader::new());
	let codemap = Rc::new(CodeMap::with_file_loader(fileLoader));
	
	let myEmitter = MessagesHandler::new(codemap.clone());
	let messages = myEmitter.messages.clone();
	let handler = Handler::with_emitter(true, false, Box::new(myEmitter));
	let sess = ParseSess::with_span_handler(handler, codemap.clone());
	
	let mut krate_result = parse_crate(source, &sess);
	
	let mut tokenWriter = tokenWriterRc.borrow_mut();
	
	if let &mut Err(ref mut db) = &mut krate_result {
		db.emit();
	}
	
	try!(tokenWriter.writeRaw("MESSAGES {\n"));
	for msg in &messages.lock().unwrap() as &Vec<SourceMessage> {
		try!(output_message(&mut tokenWriter, msg.sourcerange, &msg.message, &msg.severity));
	}
	try!(tokenWriter.writeRaw("}"));
	
	if let Ok(krate) = krate_result {
		let mut visitor : StructureVisitor = StructureVisitor::new(&codemap, &mut tokenWriter);  
		visit::walk_crate(&mut visitor, &krate);
	}
	
	Ok(())
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

pub fn parse_crate<'a>(source : &str, sess : &'a ParseSess) -> parse::PResult<'a, ast::Crate> 
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


struct MessagesHandler {
	codemap : Rc<CodeMap>,
	messages : Arc<Mutex<Vec<SourceMessage>>>,
}

use std::sync::{ Arc, Mutex };


unsafe impl ::std::marker::Send for MessagesHandler { } // FIXME: need to review this

impl MessagesHandler {
	
	fn new(codemap : Rc<CodeMap>, ) -> MessagesHandler {
		MessagesHandler { codemap : codemap, messages : Arc::new(Mutex::new(vec![])) }
	}
	
	fn writeMessage_handled(&mut self, sourcerange : Option<SourceRange>, msg: &str, severity: Severity) {
		
		let msg = SourceMessage{ severity : severity , sourcerange : sourcerange,  message : String::from(msg) };
		
		let mut messages = self.messages.lock().unwrap();
		
		messages.push(msg);
		
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


/* -----------------  ----------------- */

fn output_message(tokenWriter: &mut TokenWriter, opt_sr : Option<SourceRange>, msg: & str, lvl: &Severity) 
	-> Void
{
	
	try!(tokenWriter.out.borrow_mut().write_str("{ "));
	
	try!(outputString_Level(&lvl, tokenWriter));
	
	try!(outputString_optSourceRange(&opt_sr, tokenWriter));
	
	try!(tokenWriter.writeStringToken(msg));
	
	try!(tokenWriter.out.borrow_mut().write_str("}\n"));
	
	Ok(())
}


pub fn outputString_Level(lvl : &Severity, writer : &mut TokenWriter) -> Void {
	
	try!(lvl.output_string(&mut *writer.out.borrow_mut()));
	try!(writer.writeRaw(" "));
	
	Ok(())
}

pub fn outputString_SourceRange(sr : &SourceRange, writer : &mut TokenWriter) -> Void {
	let mut out = writer.out.borrow_mut(); 
	try!(out.write_fmt(format_args!("{{ {}:{} {}:{} }}", 
		sr.start_pos.line-1, sr.start_pos.col.0,
		sr.end_pos.line-1, sr.end_pos.col.0,
	)));
	
	Ok(())
}

pub fn outputString_optSourceRange(sr : &Option<SourceRange>, writer : &mut TokenWriter) -> Void {
	
	match sr {
		&None => try!(writer.out.borrow_mut().write_str("{ }")) ,
		&Some(ref sr) => try!(outputString_SourceRange(sr, writer)) ,
	}
	
	try!(writer.out.borrow_mut().write_str(" "));
	
	Ok(())
}
