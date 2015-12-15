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
	pub isFirstChild : bool,
	pub parentIsStruct : bool,
}

impl<'ps> StructureVisitor<'ps> {
	
	pub fn new(codemap : &'ps CodeMap, tokenWriter : &'ps mut TokenWriter) -> StructureVisitor<'ps> {
		StructureVisitor { 
			codemap : codemap, tokenWriter : tokenWriter, 
			level : 0, isFirstChild : true, parentIsStruct : false 
		}
	}
	
	pub fn writeIndent(&mut self) -> Void {
		try!(writeNTimes(&mut *self.tokenWriter.getCharOut(), ' ', self.level * 2));
		Ok(())
	}
	
	pub fn writeElement_do<FN>(&mut self, ident: &str, kind : StructureElementKind, sourceRange: &SourceRange, 
		walkFn : FN) 
		-> Void
		where FN : Fn(&mut Self) 
	{
		try!(self.tokenWriter.getCharOut().write_str("\n"));
		try!(self.writeIndent());
		
		try!(kind.writeString(&mut *self.tokenWriter.out.borrow_mut()));
		
		try!(self.tokenWriter.writeRaw(" { "));
		
		try!(self.tokenWriter.writeStringToken(ident));
		
		try!(outputString_SourceRange(sourceRange, &mut self.tokenWriter));
		
		self.level += 1;
		self.isFirstChild = true;
		walkFn(self);
		self.level -= 1;
		
		if self.isFirstChild {
			try!(self.tokenWriter.getCharOut().write_str(" "));
		} else {
			try!(self.tokenWriter.getCharOut().write_str("\n"));
			try!(self.writeIndent());
		}
		
		try!(self.tokenWriter.getCharOut().write_str("}"));
		
		self.isFirstChild = false;
		Ok(())
	}
	
	pub fn writeElement_handled<FN>(&mut self, ident: &str, kind : StructureElementKind, sourceRange: &SourceRange, 
		walkFn : FN)
		where FN : Fn(&mut Self)
	{
		use ::std::io::Write;
		
		match 
			self.writeElement_do(ident, kind, sourceRange, walkFn)
		{
			Ok(ok) => { ok } 
			Err(error) => { 
				::std::io::stderr().write_fmt(format_args!("Error writing element: {}", error)).ok(); 
			}
		}
	}
	
	pub fn writeElement<FN>(&mut self, ident: Ident, kind : StructureElementKind, span: Span, walkFn : FN)
		where FN : Fn(&mut Self)
	{
		self.writeElement_handled(&*ident.name.as_str(), kind, &SourceRange::new(self.codemap, span), walkFn)
	}
	
	/* -----------------  ----------------- */
	
	fn write_ItemUse(&mut self, vp : &ViewPath, span: Span) {
		use ::syntex_syntax::print::pprust;
		use ::syntex_syntax::ast;
		use ::std::ops::Index;
		
		let kind = StructureElementKind::Use;
		let mut useSpec = String::new();
		
		fn writePath(outString : &mut String, path : &ast::Path) {
			if path.global {
				outString.push_str("::");
			}
			outString.push_str(&pprust::path_to_string(path));
		}
		
		match &vp.node {
			&ViewPathSimple(ref ident, ref path) => {
				writePath(&mut useSpec, path);
				
				let path : &ast::Path = path;
				if path.segments.len() == 0 {
					return;
				}
				let lastSegment = path.segments.index(path.segments.len() - 1);
				if &lastSegment.identifier != ident {
					useSpec.push_str(&" as ");
					useSpec.push_str(&*ident.name.as_str());
				} 
			}
		    &ViewPathGlob(ref path) => {
		    	useSpec.push_str(&pprust::path_to_string(path));
		    	useSpec.push_str(&"::*");
		    }
		    &ViewPathList(ref path, ref pathListItem) => {
		    	writePath(&mut useSpec, path);
		    	
		    	useSpec.push_str("::{ ");
		    	for pitem in pathListItem {
		    		let rename_;
					match pitem.node {
						PathListItem_::PathListIdent{ name , rename, id : _id } => {
							useSpec.push_str(&*name.name.as_str());
							rename_ = rename; 
						}
						PathListItem_::PathListMod{ rename, id : _id } => {
							useSpec.push_str("self");
							rename_ = rename; 
						}
					};
					if rename_.is_some() {
						useSpec.push_str(" as ");
						useSpec.push_str(&*rename_.unwrap().name.as_str());
					}
					useSpec.push_str(", ");
				}
		    	useSpec.push_str("}");
		    }
		}

		self.writeElement_handled(&useSpec, kind, &SourceRange::new(self.codemap, span), 
			&|_ : &mut Self| { })
	}

}

impl<'v> Visitor<'v> for StructureVisitor<'v> {
	
	fn visit_name(&mut self, _span: Span, _name: Name) {
		// Nothing to do.
	}
	fn visit_ident(&mut self, span: Span, ident: Ident) {
		walk_ident(self, span, ident);
	}
	
	fn visit_mod(&mut self, m: &'v Mod, _span: Span, _nodeid: NodeId) {
		
//		let sr = &SourceRange::new(self.codemap, span);
//		self.writeElement_handled("_file_", StructureElementKind::File, sr, |_self : &mut Self| { 
//			walk_mod(_self, m);
//		})
		walk_mod(self, m);
	}
	
	fn visit_item(&mut self, item: &'v Item) {
		
		let kind;
		
		let noop_walkFn = &|_self : &mut Self| { };
		
		let walkFn : &Fn(&mut Self) = &|_self : &mut Self| { 
			walk_item(_self, item); 
		};
		
		match item.node {
			ItemExternCrate(_opt_name) => {
				kind = StructureElementKind::ExternCrate;
			}
			ItemUse(ref vp) => {
				self.write_ItemUse(vp, item.span);
				return;
			}
			ItemStatic(ref _typ, _, ref _expr) |
			ItemConst(ref _typ, ref _expr) => {
				self.writeElement(item.ident, StructureElementKind::Var, item.span, noop_walkFn);
				return;
			}
			ItemFn(ref declaration, unsafety, constness, abi, ref generics, ref body) => {
			    self.visit_fn(FnKind::ItemFn(item.ident, generics, unsafety, constness, abi, item.vis),
			                     declaration,
			                     body,
			                     item.span,
			                     item.id);
			    return;
			}
			ItemMod(ref _module) => {
				kind = StructureElementKind::Mod;
			}
			ItemForeignMod(ref _foreign_module) => {
				kind = StructureElementKind::Mod;
			}
			ItemTy(ref _typ, ref _type_parameters) => {
				kind = StructureElementKind::TypeAlias;
			}
			ItemEnum(ref _enum_definition, ref _type_parameters) => {
				kind = StructureElementKind::Enum;
			}
			ItemDefaultImpl(_, ref _trait_ref) => {
				// FIXME whats this?
				kind = StructureElementKind::Impl;
			}
			ItemImpl(_, _, ref _type_parameters, ref _opt_trait_reference, ref _typ, ref _impl_items) => {
	         	kind = StructureElementKind::Impl;
			}
			ItemStruct(ref _struct_definition, ref _generics) => {
				// Go straight in
				self.parentIsStruct = true;
				walk_item(self, item);
				return;
			}
			ItemTrait(_, ref _generics, ref _bounds, ref _methods) => {
				kind = StructureElementKind::Trait;
			}
			ItemMac(ref _mac) => {
				return;
			}
		}
		
		self.writeElement(item.ident, kind, item.span, walkFn);
	}
	
	fn visit_enum_def(&mut self, enum_def: &'v EnumDef, generics: &'v Generics, nodeid: NodeId, _span: Span) {
		// This element is covered by an item definition
		walk_enum_def(self, enum_def, generics, nodeid)
	}
	
	fn visit_variant(&mut self, v: &'v Variant, g: &'v Generics, nodeid: NodeId) {
		// This element is covered by an enum_def call
		walk_variant(self, v, g, nodeid);
	}
	
	fn visit_variant_data(&mut self, s: &'v VariantData, ident: Ident, _: &'v Generics, _: NodeId, span: Span) {
		let mut kind = StructureElementKind::EnumElem;
		if self.parentIsStruct {
			kind = StructureElementKind::Struct;
			self.parentIsStruct = false;
		}
		
		self.writeElement(ident, kind, span, |_self : &mut Self| { 
			walk_struct_def(_self, s);
		});
	}
	
	fn visit_struct_field(&mut self, sf: &'v StructField) {
		match sf.node.kind {
			NamedField(ident, _visibility) => {
				self.writeElement(ident, StructureElementKind::Var, sf.span, |_self : &mut Self| { 
					walk_struct_field(_self, sf); 
				});
			}
			_ => {}
		}
		
	}
	
	fn visit_trait_item(&mut self, ti: &'v TraitItem) {
		let kind;
		
	    match ti.node {
	        ConstTraitItem(ref _ty, ref _default) => {
	        	kind = StructureElementKind::Var;
	        }
	        MethodTraitItem(ref _sig, _) => {
	        	kind = StructureElementKind::Function;
	        }
	        TypeTraitItem(ref _bounds, ref _default) => {
	        	kind = StructureElementKind::TypeAlias;
	        }
	    }
	    
		self.writeElement(ti.ident, kind, ti.span, |_self : &mut Self| { 
			walk_trait_item(_self, ti); 
		});
	}
	
	fn visit_impl_item(&mut self, ii: &'v ImplItem) {
		let kind;
		
	    match ii.node {
	        ImplItemKind::Const(ref _ty, ref _default) => {
	        	kind = StructureElementKind::Var;
	        }
	        ImplItemKind::Type(ref _type)  => {
	        	kind = StructureElementKind::TypeAlias;
	        }
	        ImplItemKind::Method(_, _) |
	        ImplItemKind::Macro(_) => {
	        	walk_impl_item(self, ii);
	        	return;
	        }
	    }
		
		self.writeElement(ii.ident, kind, ii.span, |_self : &mut Self| { 
			walk_impl_item(_self, ii);
		});
	}
	
	/* ----------------- Function ----------------- */
	
	fn visit_fn(&mut self, fk: FnKind<'v>, fd: &'v FnDecl, b: &'v Block, span: Span, _nodeid: NodeId) {
		
		let ident : Ident;
		
		match fk {
		    FnKind::Method(_ident, ref _MethodSig, _option) => { 
		    	ident = _ident; 
		    }
		    FnKind::ItemFn(_ident, ref _Generics, _Unsafety, _Constness, _Abi, _Visibility) => {
		    	ident = _ident; 
		    }
		    FnKind::Closure => { return; }
		};
		
		self.writeElement(ident, StructureElementKind::Function, span, |_self : &mut Self| { 
			walk_fn(_self, fk, fd, b, span);
		});
	}
	
	fn visit_foreign_item(&mut self, foreign_item: &'v ForeignItem) { 
		let kind;
		
	    match foreign_item.node {
	        ForeignItemFn(ref _function_declaration, ref _generics) => {
	        	kind = StructureElementKind::Function;
	        }
	        ForeignItemStatic(ref _typ, _) => {
	        	kind = StructureElementKind::Var;
	        }
	    }
		
		self.writeElement(foreign_item.ident, kind, foreign_item.span, |_self : &mut Self| { 
			walk_foreign_item(_self, foreign_item); 
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
	
	fn visit_lifetime(&mut self, lifetime: &'v Lifetime) {
		walk_lifetime(self, lifetime)
	}
	fn visit_lifetime_def(&mut self, lifetime: &'v LifetimeDef) {
		walk_lifetime_def(self, lifetime)
	}
	fn visit_explicit_self(&mut self, es: &'v ExplicitSelf) {
		walk_explicit_self(self, es)
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
	fn visit_expr(&mut self, _ex: &'v Expr) {
		// Comment, no need to visit node insinde expressions 
		//walk_expr(self, ex) 
	}
	fn visit_expr_post(&mut self, _ex: &'v Expr) { 
	}
	fn visit_ty(&mut self, t: &'v Ty) { 
		walk_ty(self, t) 
	}
	fn visit_generics(&mut self, g: &'v Generics) { 
		walk_generics(self, g) 
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


#[test]
fn tests_writeStructureElement() {
	
	use ::std::rc::Rc;
	use ::std::cell::RefCell;
	
	fn test_writeStructureElement(elemName : &str, kind : StructureElementKind, sr: &SourceRange, expected : &str) {
		let stringRc = Rc::new(RefCell::new(String::new()));
		{
			let mut tw = TokenWriter { out : stringRc.clone() };
			
			let cm = CodeMap::new();
			let mut sv = StructureVisitor::new(&cm, &mut tw);
			
			sv.writeElement_do(elemName, kind, &sr, |_| { } ).ok();
		}
		
		assert_eq!(unwrapRcRefCell(stringRc).trim(), expected);
	}
	
	test_writeStructureElement("blah", StructureElementKind::Var, &sourceRange(1, 0, 2, 5), 
		r#"Var { "blah" { 1 0 2 5 } }"#);
}

#[test]
fn tests_writeStructure() {
	
	use ::std::rc::Rc;
	use ::std::cell::RefCell;
	use parse_describe;
	
	fn test_writeStructureElement(source : &str, expected : &str) {
		let stringRc = Rc::new(RefCell::new(String::new()));
		parse_describe::writeCrateStructureForSource(source, stringRc.clone());
		
		let result = unwrapRcRefCell(stringRc);
		let result = result.trim();
		if !expected.eq(result) {
			println!("{}",expected);
			println!("{}", result);
		}
		assert_eq!(result, expected);
	}
	
	test_writeStructureElement("extern crate xx;", r#"ExternCrate { "xx" { 1 0 1 16 } }"#);
	
	test_writeStructureElement("const xx : u32 = 1;", r#"Var { "xx" { 1 0 1 19 } }"#);
	
	
	test_writeStructureElement("mod myMod   ;  ", r#"Mod { "myMod" { 1 0 1 13 } }"#);
	test_writeStructureElement("mod myMod { }", r#"Mod { "myMod" { 1 0 1 13 } }"#);
	test_writeStructureElement("mod myMod { static xx : u32 = 2; }", 
r#"Mod { "myMod" { 1 0 1 34 }
  Var { "xx" { 1 12 1 32 } }
}"#
	);
	
	test_writeStructureElement("fn xx(a : &str) -> u32 { }", r#"Function { "xx" { 1 0 1 26 } }"#);
	
	test_writeStructureElement("type MyType = &u32<asd>;", r#"TypeAlias { "MyType" { 1 0 1 24 } }"#);
	
	test_writeStructureElement("enum MyEnum { Alpha, Beta, } ", 
r#"Enum { "MyEnum" { 1 0 1 28 }
  EnumElem { "Alpha" { 1 14 1 19 } }
  EnumElem { "Beta" { 1 21 1 25 } }
}"#);
	test_writeStructureElement("enum MyEnum<T, U> { Alpha(T), Beta(U), } ", 
r#"Enum { "MyEnum" { 1 0 1 40 }
  EnumElem { "Alpha" { 1 20 1 28 } }
  EnumElem { "Beta" { 1 30 1 37 } }
}"#);
	
	
	test_writeStructureElement("struct MyStruct ( u32, blah<sdf> ); ", r#"Struct { "MyStruct" { 1 0 1 35 } }"#);
	test_writeStructureElement("struct MyStruct { foo : u32, } ", 
r#"Struct { "MyStruct" { 1 0 1 30 }
  Var { "foo" { 1 18 1 27 } }
}"#);
	
	test_writeStructureElement("trait MyTrait { } ", r#"Trait { "MyTrait" { 1 0 1 17 } }"#);
	test_writeStructureElement("trait MyTrait : Foo { fn xxx(); } ", 
r#"Trait { "MyTrait" { 1 0 1 33 }
  Function { "xxx" { 1 22 1 31 } }
}"#);
	test_writeStructureElement("trait MyTrait : Foo { type N: fmt::Display; fn xxx(); const foo :u32 = 3; } ", 
r#"Trait { "MyTrait" { 1 0 1 75 }
  TypeAlias { "N" { 1 22 1 43 } }
  Function { "xxx" { 1 44 1 53 } }
  Var { "foo" { 1 54 1 73 } }
}"#);
	
	test_writeStructureElement("impl MyType { } ", r#"Impl { "MyType" { 1 0 1 15 } }"#);
	test_writeStructureElement("impl MyTrait for MyType { } ", r#"Impl { "MyType.MyTrait" { 1 0 1 27 } }"#);
	test_writeStructureElement("impl  MyTrait       { type N= fmt::Display; fn xx(){} const foo :u32 = 3; } ", 
r#"Impl { "MyTrait" { 1 0 1 75 }
  TypeAlias { "N" { 1 22 1 43 } }
  Function { "xx" { 1 44 1 53 } }
  Var { "foo" { 1 54 1 73 } }
}"#);	
	

	test_writeStructureElement("use blah;", r#"Use { "blah" { 1 0 1 9 } }"#);
	test_writeStructureElement("use blah as foo;", r#"Use { "blah as foo" { 1 0 1 16 } }"#);
	// TODO: this is not printing the global path prefix, seems to be a limitation from libsyntax?
	test_writeStructureElement("use ::blah::foo as myfoo;", r#"Use { "blah::foo as myfoo" { 1 0 1 25 } }"#);
	test_writeStructureElement("use ::blah::foo::*;", r#"Use { "blah::foo::*" { 1 0 1 19 } }"#);
	test_writeStructureElement("use blah::foo:: { One as OtherOne, self as Two };", 
		r#"Use { "blah::foo::{ One as OtherOne, self as Two, }" { 1 0 1 49 } }"#);
	
	
	test_writeStructureElement("my_macro!(asf); ", "");
	
	test_writeStructureElement("macro_rules! foo { (x => $e:expr) => (); }", "");
	
	
	test_writeStructureElement("extern { fn ext(p : u32); }", 
r#"Mod { "" { 1 0 1 27 }
  Function { "ext" { 1 9 1 25 } }
}"#);
	test_writeStructureElement("extern { fn ext(p : u32); \n static extVar: u8; }", 
r#"Mod { "" { 1 0 2 21 }
  Function { "ext" { 1 9 1 25 } }
  Var { "extVar" { 2 1 2 19 } }
}"#);
	
	// Test with a lexer error, FIXME
	//test_writeStructureElement("const xx : u32 = '", r#"Var { "xx" { 1 0 1 19 } }"#);	
}