use ::core_util::*;
use ::misc_util::*;
use ::ranges::*;

use ::syntex_syntax::syntax::ast::*;
use ::syntex_syntax::parse::{ self, ParseSess };
use ::syntex_syntax::visit;
use ::syntex_syntax::codemap:: { self, Span, Loc, CodeMap};
use ::syntex_syntax::diagnostic:: { self, SpanHandler, Handler, RenderSpan, Level };

use std::boxed::Box;
use std::io;
use std::io::Write;

use ::token_writer::TokenWriter;

//use std::cell::*;
use std::rc::*;

pub fn parse_analysis(source : &str) {
	
	let tokenWriter = TokenWriter { out : Box::new(StdoutWrite(io::stdout())) };
	let mut tokenWriter = Rc::new(tokenWriter);
	
	println!("RUST_PARSE_DESCRIBE 0.1");
	
	let krate_result : parse::PResult<Crate>;
	let codemap;
	
	{	
		let myEmitter = MessagesHandler { tokenWriter : tokenWriter.clone() };
		let handler = Handler::with_emitter(true, Box::new(myEmitter));
		let spanhandler = SpanHandler::new(handler, CodeMap::new());
		let sess = ParseSess::with_span_handler(spanhandler);
		
		let cfg = vec![];
		
		let krateName = "name".to_string(); // XXX: mod name
		
		krate_result = 
			parse::new_parser_from_source_str(&sess, cfg, krateName, source.to_string())
			.parse_crate_mod();
		
		codemap = sess.span_diagnostic.cm;
	}	
	
	let tokenWriter = Rc::get_mut(&mut tokenWriter).unwrap();
	
	writeCrateStructure(&codemap, &krate_result, tokenWriter);
}

struct MessagesHandler {
	tokenWriter: Rc<TokenWriter>,
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
		
		let mut tokenWriter = Rc::get_mut(&mut self.tokenWriter).unwrap();
		try!(tokenWriter.out.write_str("MESSAGE { "));
		
		try!(outputString_Level(&lvl, &mut tokenWriter));
		
		try!(outputString_optSourceRange(&sourcerange, &mut tokenWriter));
		
		try!(tokenWriter.writeStringToken(msg));
		
		try!(tokenWriter.out.write_str("}\n"));
		
		Ok(())
	}
}


	fn outputString_Level(lvl : &Level, writer : &mut TokenWriter) -> Void {
		let str = match *lvl {
			Level::Bug => panic!("Bug parsing error code"),
			Level::Fatal => "error",
			Level::Error => "error",
			Level::Warning => "warning",
			Level::Note => "note",
			Level::Help => "help",
		};
		
		try!(writer.out.write_str(str));
		try!(writer.out.write_str(" "));
		
		Ok(())
	}
	
	fn outputString_SourceRange(sr : &SourceRange, writer : &mut TokenWriter) -> Void {
		
		try!(writer.out.0.write_fmt(format_args!("{{ {} {} {} {} }}", 
			sr.start_pos.line, sr.start_pos.col.0,
			sr.end_pos.line, sr.start_pos.col.0,
		)));
		
		Ok(())
	}
	
	fn outputString_optSourceRange(sr : &Option<SourceRange>, writer : &mut TokenWriter) -> Void {
		
		match sr {
			&None => try!(writer.out.write_str("{ }")) ,
			&Some(ref sr) => try!(outputString_SourceRange(sr, writer)) ,
		}
		
		try!(writer.out.write_str(" "));
		
		Ok(())
	}	




pub fn writeCrateStructure(codemap : &CodeMap, krate_result : &parse::PResult<Crate>, tokenWriter : &mut TokenWriter) {

	io::stdout().flush().unwrap();
	
	let krate = match krate_result {
		&Err(err) => { 
			io::stderr().write_fmt(format_args!("Error parsing source: {}\n", err)).unwrap(); 
			return; 
		}
		&Ok(ref ok_krate) => { ok_krate }
	};
	
	let mut visitor : StructureVisitor = StructureVisitor { codemap : codemap, level : 0 };  
	
	visit::walk_crate(&mut visitor, &krate);
	
}

struct StructureVisitor<'ps> {
	codemap : & 'ps CodeMap,
	level : u32,
}

impl<'ps> StructureVisitor<'ps> {
	
	fn previsit(&mut self, span: Span, _ : NodeId) {
		print_span(&span, self.codemap);
	}
	
}


mod structure_visitor {
	
	use super::StructureVisitor;
	use ::syntex_syntax::visit::*;
	use ::syntex_syntax::ast::*;
	use ::syntex_syntax::codemap:: { Span, Loc };
	
	impl<'v> Visitor<'v> for StructureVisitor<'v> {
		
		
	fn visit_name(&mut self, span: Span, _name: Name) {
		// Nothing to do.
	}
	fn visit_ident(&mut self, span: Span, ident: Ident) {
		walk_ident(self, span, ident);
	}
	fn visit_mod(&mut self, m: &'v Mod, span: Span, nodeid: NodeId) {
		self.previsit(span, nodeid);
		 
		walk_mod(self, m) 
	}
	
	
	fn visit_foreign_item(&mut self, i: &'v ForeignItem) { 
		walk_foreign_item(self, i) 
	}
	fn visit_item(&mut self, i: &'v Item) { 
		walk_item(self, i) 
	}
	fn visit_local(&mut self, l: &'v Local) { 
		walk_local(self, l) 
	}
	fn visit_block(&mut self, b: &'v Block) { 
		walk_block(self, b) 
	}
	fn visit_stmt(&mut self, s: &'v Stmt) { 
		walk_stmt(self, s) 
	}
	fn visit_arm(&mut self, a: &'v Arm) { 
		walk_arm(self, a) 
	}
	fn visit_pat(&mut self, p: &'v Pat) { 
		walk_pat(self, p) 
	}
	fn visit_decl(&mut self, d: &'v Decl) { 
		walk_decl(self, d) 
	}
	fn visit_expr(&mut self, ex: &'v Expr) { 
		walk_expr(self, ex) 
	}
	fn visit_expr_post(&mut self, _ex: &'v Expr) { }
	fn visit_ty(&mut self, t: &'v Ty) { 
		walk_ty(self, t) 
	}
	fn visit_generics(&mut self, g: &'v Generics) { 
		walk_generics(self, g) 
	}
	fn visit_fn(&mut self, fk: FnKind<'v>, fd: &'v FnDecl, b: &'v Block, span: Span, nodeid: NodeId) {
		self.previsit(span, nodeid);
		
		walk_fn(self, fk, fd, b, span)
	}
	fn visit_trait_item(&mut self, ti: &'v TraitItem) {
		self.previsit(ti.span, ti.id);
	
		walk_trait_item(self, ti) 
	}
	fn visit_impl_item(&mut self, ii: &'v ImplItem) { 
		self.previsit(ii.span, ii.id);
		
		walk_impl_item(self, ii) 
	}
	fn visit_trait_ref(&mut self, t: &'v TraitRef) { 
		walk_trait_ref(self, t) 
	}
	fn visit_ty_param_bound(&mut self, bounds: &'v TyParamBound) {
		walk_ty_param_bound(self, bounds)
	}
	fn visit_poly_trait_ref(&mut self, t: &'v PolyTraitRef, m: &'v TraitBoundModifier) {
		walk_poly_trait_ref(self, t, m)
	}
	fn visit_variant_data(&mut self, s: &'v VariantData, _: Ident, _: &'v Generics, nodeid: NodeId, span: Span) {
		self.previsit(span, nodeid);
		
		walk_struct_def(self, s)
	}
	fn visit_struct_field(&mut self, sf: &'v StructField) { 
		self.previsit(sf.span, sf.node.id);
		
		walk_struct_field(self, sf) 
	}
	fn visit_enum_def(&mut self, enum_def: &'v EnumDef, generics: &'v Generics, nodeid: NodeId, span: Span) {
		self.previsit(span, nodeid);
		
		walk_enum_def(self, enum_def, generics, nodeid)
	}
	fn visit_variant(&mut self, v: &'v Variant, g: &'v Generics, nodeid: NodeId) {
		self.previsit(v.span, nodeid); // FIXME: review
		
		walk_variant(self, v, g, nodeid)
	}
	
	fn visit_lifetime(&mut self, lifetime: &'v Lifetime) {
		walk_lifetime(self, lifetime)
	}
	fn visit_lifetime_def(&mut self, lifetime: &'v LifetimeDef) {
		walk_lifetime_def(self, lifetime)
	}
	fn visit_explicit_self(&mut self, es: &'v ExplicitSelf) {
		walk_explicit_self(self, es)
	}
	
	fn visit_mac(&mut self, _mac: &'v Mac) {
		panic!("visit_mac disabled by default");
		// NB: see note about macros above.
		// if you really want a visitor that
		// works on macros, use this
		// definition in your trait impl:
		// visit::walk_mac(self, _mac)
	}
	fn visit_path(&mut self, path: &'v Path, _id: NodeId) {
		walk_path(self, path)
	}
	fn visit_path_list_item(&mut self, prefix: &'v Path, item: &'v PathListItem) {
		walk_path_list_item(self, prefix, item)
	}
	fn visit_path_segment(&mut self, path_span: Span, path_segment: &'v PathSegment) {
		walk_path_segment(self, path_span, path_segment)
	}
	fn visit_path_parameters(&mut self, path_span: Span, path_parameters: &'v PathParameters) {
		walk_path_parameters(self, path_span, path_parameters)
	}
	fn visit_assoc_type_binding(&mut self, type_binding: &'v TypeBinding) {
		walk_assoc_type_binding(self, type_binding)
	}
	fn visit_attribute(&mut self, _attr: &'v Attribute) {
	}
	fn visit_macro_def(&mut self, macro_def: &'v MacroDef) {
		self.previsit(macro_def.span, macro_def.id);
		
		walk_macro_def(self, macro_def)
	}
		
	}

}

/* -----------------  ----------------- */

fn print_span(span : &Span, codemap : &CodeMap) {
	let start_pos = codemap.lookup_char_pos(span.lo);
	let end_pos = codemap.lookup_char_pos(span.hi);
	print_locs(&start_pos, &end_pos);
}

fn print_locs(start_loc : &Loc, end_loc : &Loc) {
	println!("Span: {0}:{1} - {2}:{3}", start_loc.line, start_loc.col.0, end_loc.line, end_loc.col.0);
}