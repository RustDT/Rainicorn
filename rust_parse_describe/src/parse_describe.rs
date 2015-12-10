use ::util::core::*;
use ::util::string::*;
use ::ranges::*;

use ::syntex_syntax::syntax::ast::*;
use ::syntex_syntax::parse::{ self, ParseSess };
use ::syntex_syntax::visit;
use ::syntex_syntax::codemap:: { self, Span, CodeMap};
use ::syntex_syntax::diagnostic:: { self, SpanHandler, Handler, RenderSpan, Level };

use std::boxed::Box;
use std::io;
use std::io::Write;

use ::token_writer::TokenWriter;

use std::cell::RefCell;
use std::rc::*;


pub fn parse_analysis(source : &str) {
	
	let tokenWriter = TokenWriter { out : Rc::new(RefCell::new(StdoutWrite(io::stdout()))) };
	let tokenWriterRc : Rc<RefCell<TokenWriter>> = Rc::new(RefCell::new(tokenWriter));
	
	println!("RUST_PARSE_DESCRIBE 0.1");
	
	let (krate_result, codemap) = parse_crate(source, tokenWriterRc.clone()); 
	
	let mut tokenWriter = unwrapRcRefCell(tokenWriterRc);
	
	writeCrateStructure(&codemap, &krate_result, &mut tokenWriter);
}

pub fn parse_crate(source : &str, tokenWriter : Rc<RefCell<TokenWriter>>) -> (parse::PResult<Crate>, CodeMap) {
	let myEmitter = MessagesHandler { tokenWriter : tokenWriter };
	let handler = Handler::with_emitter(true, Box::new(myEmitter));
	let spanhandler = SpanHandler::new(handler, CodeMap::new());
	let sess = ParseSess::with_span_handler(spanhandler);
	
	let cfg = vec![];
	
	let krateName = "_file_module_".to_string();
	
	let krate_result = 
		parse::new_parser_from_source_str(&sess, cfg, krateName, source.to_string())
		.parse_crate_mod();
	
	return (krate_result, sess.span_diagnostic.cm);
}

struct MessagesHandler {
	tokenWriter: Rc<RefCell<TokenWriter>>,
}


unsafe impl ::std::marker::Send for MessagesHandler { } // FIXME: need to review this

impl diagnostic::Emitter for MessagesHandler {
	
    fn emit(&mut self, cmsp: Option<(&codemap::CodeMap, Span)>, msg: &str, code: Option<&str>, lvl: Level) {
    	
    	match self.outputMessage(cmsp, msg, code, lvl) {
    		Ok(_) => {}
    		Err(err) => {
    			io::stderr().write_fmt(format_args!("Error serializing compiler message: {}\n", err)).unwrap();
			}
    	}
    	
    }
    
    fn custom_emit(&mut self, _: &codemap::CodeMap, _: RenderSpan, _: &str, _: Level) {
    	panic!("custom_emit called!");
    }
	
}

impl MessagesHandler {
	
	fn outputMessage(&mut self, cmsp: Option<(&codemap::CodeMap, Span)>, msg: &str, _: Option<&str>, lvl: Level) 
		-> Void
	{
		let sourcerange = match cmsp {
			Some((codemap, span)) => Some(SourceRange::new(codemap, span)),
			None => None,
		};
		
		let mut tokenWriter = &mut self.tokenWriter.borrow_mut();
		try!(tokenWriter.out.borrow_mut().write_str("MESSAGE { "));
		
		try!(outputString_Level(&lvl, &mut tokenWriter));
		
		try!(outputString_optSourceRange(&sourcerange, &mut tokenWriter));
		
		try!(tokenWriter.writeStringToken(msg));
		
		try!(tokenWriter.out.borrow_mut().write_str("}\n"));
		
		Ok(())
	}
}


use ::structure_visitor::StructureVisitor;

pub fn writeCrateStructure(codemap : &CodeMap, krate_result : &parse::PResult<Crate>, tokenWriter : &mut TokenWriter) {

	io::stdout().flush().unwrap();
	
	let krate = match krate_result {
		&Err(err) => { 
			io::stderr().write_fmt(format_args!("Error parsing source: {}\n", err)).unwrap(); 
			return; 
		}
		&Ok(ref ok_krate) => { ok_krate }
	};
	
	let mut visitor : StructureVisitor = StructureVisitor::new(codemap, tokenWriter);  
	
	visit::walk_crate(&mut visitor, &krate);
	
}


/* -----------------  ----------------- */


pub fn outputString_Level(lvl : &Level, writer : &mut TokenWriter) -> Void {
	let str = match *lvl {
		Level::Bug => panic!("Bug parsing error code"),
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
