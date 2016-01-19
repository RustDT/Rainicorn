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
use ::ranges::*;

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
	
	let myEmitter = MessagesHandler { tokenWriter : tokenWriterRc.clone() , codemap : codemap.clone()};
	let handler = Handler::with_emitter(true, true , Box::new(myEmitter));
	let sess = ParseSess::with_span_handler(handler, codemap.clone());
	
	try!(tokenWriterRc.borrow_mut().writeRaw("MESSAGES {\n"));
	let krate_result = parse_crate(source, &sess);
	try!(tokenWriterRc.borrow_mut().writeRaw("}"));
	
	let mut tokenWriter = tokenWriterRc.borrow_mut();
	
	match krate_result {
		Err(_err) => {
			// Error messages should have been written to out
		}
		Ok(ref krate) => { 
			let mut visitor : StructureVisitor = StructureVisitor::new(&codemap, &mut tokenWriter);  
			visit::walk_crate(&mut visitor, &krate);
		}
	};
	
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
	let cfg = vec![];
	let krateName = "_file_module_".to_string();
	
	return parse::new_parser_from_source_str(&sess, cfg, krateName, source.to_string()).parse_crate_mod();
}


struct MessagesHandler {
	tokenWriter: Rc<RefCell<TokenWriter>>,
	codemap : Rc<CodeMap>,
}


unsafe impl ::std::marker::Send for MessagesHandler { } // FIXME: need to review this

impl MessagesHandler {
	
	fn writeMessage_handled(&mut self, sourcerange : Option<SourceRange>, msg: &str, lvl: Level) {
		match self.outputMessage(sourcerange, msg, lvl) {
    		Ok(_) => {}
    		Err(err) => {
    			io::stderr().write_fmt(format_args!("Error serializing compiler message: {}\n", err)).ok();
    			io::stderr().flush().ok();
			}
    	}
	}
	
}

impl emitter::Emitter for MessagesHandler {
	
    fn emit(&mut self, cmsp: Option<Span>, msg: &str, code: Option<&str>, lvl: Level) {
    	
    	match code {
    		None => {}
    		Some(code) => {
    			io::stderr().write_fmt(format_args!("Code: {}\n", code)).unwrap();
    			panic!("What is code: Option<&str>??");
			}
    	}
    	
    	
		let sourcerange = match cmsp {
			Some(span) => Some(SourceRange::new(&self.codemap, span)),
			None => None,
		};
		
		self.writeMessage_handled(sourcerange, msg, lvl);
    }
    
    fn custom_emit(&mut self, _: RenderSpan, msg: &str, lvl: Level) {
    	if match lvl { Level::Help | Level::Note => true, _ => false } {
    		return;
    	}
    	
    	self.writeMessage_handled(None, msg, lvl);
    }
	
}

impl MessagesHandler {
	
	fn outputMessage(&mut self, opt_sr : Option<SourceRange>, msg: &str, lvl: Level) 
		-> Void
	{
		
		let mut tokenWriter = &mut self.tokenWriter.borrow_mut();
		try!(tokenWriter.out.borrow_mut().write_str("MESSAGE { "));
		
		try!(outputString_Level(&lvl, &mut tokenWriter));
		
		try!(outputString_optSourceRange(&opt_sr, &mut tokenWriter));
		
		try!(tokenWriter.writeStringToken(msg));
		
		try!(tokenWriter.out.borrow_mut().write_str("}\n"));
		
		Ok(())
	}
}



/* -----------------  ----------------- */


pub fn outputString_Level(lvl : &Level, writer : &mut TokenWriter) -> Void {
	let str = match *lvl {
		Level::Bug => panic!("Bug parsing error code"),
		Level::Cancelled => "cancelled",
		Level::Fatal => "error",
		Level::Error => "error",
		Level::Warning => "warning",
		Level::Note => "note",
		Level::Help => "help",
	};
	
	try!(writer.out.borrow_mut().write_str(str));
	try!(writer.out.borrow_mut().write_str(" "));
	
	Ok(())
}

pub fn outputString_SourceRange(sr : &SourceRange, writer : &mut TokenWriter) -> Void {
	let mut out = writer.out.borrow_mut(); 
	try!(out.write_fmt(format_args!("{{ {} {} {} {} }}", 
		sr.start_pos.line, sr.start_pos.col.0,
		sr.end_pos.line, sr.end_pos.col.0,
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
