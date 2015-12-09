//!
//! Write a parse structure into a TokenWriter
//! 
	
use ::util::core::*;
use ::util::string::*;
use ::ranges::*;

use ::syntex_syntax::visit::*;
use ::syntex_syntax::ast::*;
use ::syntex_syntax::codemap:: { Span, CodeMap };

use ::token_writer::TokenWriter;
use ::parse_describe::*;


pub struct StructureVisitor<'ps> {
	pub codemap : & 'ps CodeMap,
	pub tokenWriter : &'ps mut TokenWriter,
	pub level : u32,
}

impl<'ps> StructureVisitor<'ps> {
	
	pub fn new(codemap : &'ps CodeMap, tokenWriter : &'ps mut TokenWriter) -> StructureVisitor<'ps> {
		StructureVisitor { codemap : codemap, tokenWriter : tokenWriter, level : 0 }
	}
	
	pub fn writeElement_do<FN>(&mut self, ident: &str, sourceRange: &SourceRange, walkFn : FN) 
		where FN : Fn(&mut Self)
	{
//		self.tokenWriter.out.write_str("\n");
		writeNTimes(&mut *self.tokenWriter.out, ' ', self.level);
		
		self.tokenWriter.writeTextToken("ITEM");
		self.tokenWriter.writeTextToken(" { ");
		
		self.tokenWriter.writeStringToken(ident);
		
		outputString_SourceRange(sourceRange, &mut self.tokenWriter);
		
		self.level += 1;
		walkFn(self);
		self.level -= 1;
		
//		self.tokenWriter.out.write_str("\n");
		writeNTimes(&mut *self.tokenWriter.out, ' ', self.level);
		self.tokenWriter.out.write_str(" }\n");
	}
	
	
	pub fn writeElement<FN>(&mut self, ident: Ident, span: Span, walkFn : FN) 
		where FN : Fn(&mut Self)
	{
		self.writeElement_do(&*ident.name.as_str(), &SourceRange::new(self.codemap, span), walkFn);
	}
	
}

impl<'v> Visitor<'v> for StructureVisitor<'v> {
		
		
	fn visit_name(&mut self, _span: Span, _name: Name) {
		// Nothing to do.
	}
	fn visit_ident(&mut self, span: Span, ident: Ident) {
		walk_ident(self, span, ident);
	}
	
	fn visit_mod(&mut self, m: &'v Mod, span: Span, _nodeid: NodeId) {
		let ident = Ident::with_empty_ctxt(Name(12));
		
		self.writeElement_do("_file_", &SourceRange::new(self.codemap, span), |_self : &mut Self| { 
			walk_mod(_self, m);
		});
	}
	
	
	fn visit_foreign_item(&mut self, item: &'v ForeignItem) { 
		self.writeElement(item.ident, item.span, |_self : &mut Self| { 
			walk_foreign_item(_self, item); 
		});
	}
	fn visit_item(&mut self, item: &'v Item) { 
		self.writeElement(item.ident, item.span, |_self : &mut Self| { 
			walk_item(_self, item); 
		});
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
//		walk_expr(self, ex) 
	}
	fn visit_expr_post(&mut self, _ex: &'v Expr) { 
	}
	fn visit_ty(&mut self, t: &'v Ty) { 
		walk_ty(self, t) 
	}
	fn visit_generics(&mut self, g: &'v Generics) { 
		walk_generics(self, g) 
	}
	
	fn visit_fn(&mut self, fk: FnKind<'v>, fd: &'v FnDecl, b: &'v Block, span: Span, nodeid: NodeId) {
		
		let ident : Ident;
		
		match fk {
//		    ItemFn(ident, generics, unsafety, constness, abi, visibility) => {}
			
		    FnKind::Method(_ident, ref MethodSig, option) => { 
		    	ident = _ident; 
		    }
		    FnKind::ItemFn(_ident, ref Generics, Unsafety, Constness, Abi, Visibility) => {
		    	ident = _ident; 
		    }
		    FnKind::Closure => { return; }
		};
		
		self.writeElement(ident, span, |_self : &mut Self| { 
			walk_fn(_self, fk, fd, b, span);
		});
	}
	
	fn visit_trait_item(&mut self, ti: &'v TraitItem) {
		self.writeElement(ti.ident, ti.span, |_self : &mut Self| { 
			walk_trait_item(_self, ti); 
		});
	}
	
	fn visit_impl_item(&mut self, ii: &'v ImplItem) { 
		self.writeElement(ii.ident, ii.span, |_self : &mut Self| { 
			walk_impl_item(_self, ii);
		});
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
	fn visit_variant_data(&mut self, s: &'v VariantData, ident: Ident, _: &'v Generics, _: NodeId, span: Span) {
		self.writeElement(ident, span, |_self : &mut Self| { 
			walk_struct_def(_self, s);
		});
	}
	fn visit_struct_field(&mut self, sf: &'v StructField) { 
//		self.writeElement(sf.ident, sf.span, |_self : &mut Self| { 
			walk_struct_field(self, sf); 
//		});
	}
		
	fn visit_enum_def(&mut self, enum_def: &'v EnumDef, generics: &'v Generics, nodeid: NodeId, span: Span) {
//		self.writeElement(ident, span, |_self : &mut Self| { 
			walk_enum_def(self, enum_def, generics, nodeid)
//		});
	}
	
	fn visit_variant(&mut self, v: &'v Variant, g: &'v Generics, nodeid: NodeId) {
		self.writeElement(v.node.name, v.span, |_self : &mut Self| { 
			walk_variant(_self, v, g, nodeid);
		});
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
		walk_macro_def(self, macro_def)
	}
	
}