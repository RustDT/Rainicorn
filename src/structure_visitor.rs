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

use util::core::*;
use source_model::*;

use syntex_syntax::visit::*;
use syntex_syntax::ast::*;
use syntex_syntax::codemap:: { Span, CodeMap };

pub struct StructureVisitor<'ps> {
    pub codemap : & 'ps CodeMap,
    pub parentIsStruct : bool,
    
    pub elements : Vec<StructureElement>,
}

impl<'ps> StructureVisitor<'ps> {
    
    pub fn new(codemap : &'ps CodeMap) -> StructureVisitor<'ps> {
        StructureVisitor { 
            codemap : codemap, parentIsStruct : false, elements : vec![]
        }
    }
    
    pub fn writeElement_do<FN>(&mut self, ident: &str, kind: StructureElementKind, sourcerange: SourceRange,
        type_desc: String,  
        walkFn: FN) 
        -> Void
        where FN : Fn(&mut Self) 
    {

        let mut siblings = vec![];
        ::std::mem::swap(&mut self.elements, &mut siblings);
        
        walkFn(self); // self.elements now has children
        
        ::std::mem::swap(&mut self.elements, &mut siblings);
        let children = siblings;
        
        let element = StructureElement{ name: String::from(ident), kind: kind, sourcerange: sourcerange ,
            type_desc : type_desc,
            children : children };
        
        self.elements.push(element);
        Ok(())
    }
    
    pub fn writeElement_handled<FN>(&mut self, ident: &str, kind : StructureElementKind, sourceRange: SourceRange, 
        type_desc: String,
        walkFn : FN)
        where FN : Fn(&mut Self)
    {
        use std::io::Write;
        
        match 
            self.writeElement_do(ident, kind, sourceRange, type_desc, walkFn)
        {
            Ok(ok) => { ok } 
            Err(error) => { 
                ::std::io::stderr().write_fmt(format_args!("Error writing element: {}", error)).ok(); 
            }
        }
    }
    
    pub fn writeElement_TODO<FN>(&mut self, ident: Ident, kind : StructureElementKind, span: Span, 
        walkFn : FN)
        where FN : Fn(&mut Self)
    {
        self.writeElement(ident, kind, span, "".to_string(), walkFn);
    }
    
    pub fn writeElement<FN>(&mut self, ident: Ident, kind : StructureElementKind, span: Span, 
        type_desc: String, walkFn : FN)
        where FN : Fn(&mut Self)
    {
        self.writeElement_handled(&*ident.name.as_str(), kind, SourceRange::new(self.codemap, span), 
            type_desc, walkFn)
    }
    
    /* -----------------  ----------------- */
    
    fn write_ItemUse(&mut self, vp : &ViewPath, span: Span) {
        use syntex_syntax::print::pprust;
        use syntex_syntax::ast;
        use std::ops::Index;
        
        let kind = StructureElementKind::Use;
        let mut useSpec = String::new();
        
        fn writePath(outString : &mut String, path : &ast::Path) {
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
                        PathListItemKind::Ident{ name , rename, id : _id } => {
                            useSpec.push_str(&*name.name.as_str());
                            rename_ = rename; 
                        }
                        PathListItemKind::Mod{ rename, id : _id } => {
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

        self.writeElement_handled(&useSpec, kind, SourceRange::new(self.codemap, span), "".to_string(), 
            &|_ : &mut Self| { })
    }
    
    fn get_type_desc_from_fndecl(&mut self, fd: &FnDecl) -> String {
        
        let mut type_desc = "".to_string();
        
        type_desc.push('(');
        let mut needs_sep = false;
        
        for arg in &fd.inputs {
            let arg : &Arg = arg;
//            let pat : &Pat_ = &arg.pat.node;
            let pat_span = arg.pat.span;
            
            if needs_sep {
                type_desc.push_str(", ");
            }
            needs_sep = true;
            
            if let Ok(snippet) = self.codemap.span_to_snippet(pat_span) {
                if snippet == "self" {
                    type_desc.push_str(&snippet);
                    continue;
                }
            }
            
            if let Ok(snippet) = self.codemap.span_to_snippet(arg.ty.span) {
                type_desc.push_str(&snippet);
            }
        };
        type_desc.push_str(")");
        
        if let FunctionRetTy::Ty(ref _ret) = fd.output {
            if let Ok(ret_snippet) = self.codemap.span_to_snippet(fd.output.span()) {
                type_desc.push_str(" -> ");
                type_desc.push_str(&ret_snippet);
            }
        }
        
        type_desc
    }
    
    fn write_function_element(&mut self, ident: Ident, span: Span, fd: & FnDecl, walkFn : &Fn(&mut Self)) {
        let type_desc = self.get_type_desc_from_fndecl(&fd);
        
        self.writeElement(ident, StructureElementKind::Function, span, type_desc, walkFn);
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
        
//        let sr = &SourceRange::new(self.codemap, span);
//        self.writeElement_handled("_file_", StructureElementKind::File, sr, |_self : &mut Self| { 
//            walk_mod(_self, m);
//        })
        walk_mod(self, m);
    }
    
    fn visit_item(&mut self, item: &'v Item) {
        
        let kind;
        let mut type_desc = "".to_string();
        
        let noop_walkFn = &|_self : &mut Self| { };
        
        let walkFn : &Fn(&mut Self) = &|_self : &mut Self| { 
            walk_item(_self, item); 
        };
        
        match item.node {
            ItemKind::ExternCrate(_opt_name) => {
                kind = StructureElementKind::ExternCrate;
            }
            ItemKind::Use(ref vp) => {
                self.write_ItemUse(vp, item.span);
                return;
            }
            ItemKind::Static(ref typ, _, ref _expr) |
            ItemKind::Const(ref typ, ref _expr) => {
                
                if let Ok(snippet) = self.codemap.span_to_snippet(typ.span) {
                    type_desc.push_str(&snippet);
                }
                self.writeElement(item.ident, StructureElementKind::Var, item.span, type_desc, noop_walkFn);
                return;
            }
            ItemKind::Fn(ref declaration, unsafety, constness, abi, ref generics, ref body) => {
                self.visit_fn(FnKind::ItemFn(item.ident, generics, unsafety, constness, abi, &item.vis),
                                 declaration,
                                 body,
                                 item.span,
                                 item.id);
                return;
            }
            ItemKind::Mod(ref _module) => {
                kind = StructureElementKind::Mod;
            }
            ItemKind::ForeignMod(ref _foreign_module) => {
                kind = StructureElementKind::Mod;
            }
            ItemKind::Ty(ref _typ, ref _type_parameters) => {
                kind = StructureElementKind::TypeAlias;
            }
            ItemKind::Enum(ref _enum_definition, ref _type_parameters) => {
                kind = StructureElementKind::Enum;
            }
            ItemKind::DefaultImpl(_, ref _trait_ref) => {
                kind = StructureElementKind::Impl;
            }
            ItemKind::Impl(_, _, ref _type_parameters, ref _opt_trait_reference, ref _typ, ref _impl_items) => {
                 kind = StructureElementKind::Impl;
            }
            ItemKind::Struct(ref _struct_definition, ref _generics) => {
                // Go straight in
                self.parentIsStruct = true;
                walk_item(self, item);
                return;
            }
            ItemKind::Trait(_, ref _generics, ref _bounds, ref _methods) => {
                kind = StructureElementKind::Trait;
            }
            ItemKind::Mac(ref _mac) => {
                return;
            }
        }
        
        self.writeElement(item.ident, kind, item.span, type_desc, walkFn);
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
        let mut kind = StructureElementKind::EnumVariant;
        if self.parentIsStruct {
            kind = StructureElementKind::Struct;
            self.parentIsStruct = false;
        }
        
        self.writeElement_TODO(ident, kind, span, |_self : &mut Self| { 
            walk_struct_def(_self, s);
        });
    }
    
    fn visit_struct_field(&mut self, sf: &'v StructField) {
        if let Some(ident) = sf.ident {
            self.writeElement_TODO(ident, StructureElementKind::Var, sf.span, |_self : &mut Self| { 
                walk_struct_field(_self, sf); 
            });
        }
    }
    
    fn visit_trait_item(&mut self, ti: &'v TraitItem) {
        let kind;
        
        match ti.node {
            TraitItemKind::Const(ref _ty, ref _default) => {
                kind = StructureElementKind::Var;
            }
            TraitItemKind::Method(ref sig, _) => {
                self.write_function_element(ti.ident, ti.span, &sig.decl, &|_self : &mut Self| { 
                    walk_trait_item(_self, ti);
                });
                return;
            }
            TraitItemKind::Type(ref _bounds, ref _default) => {
                kind = StructureElementKind::TypeAlias;
            }
        }
        
        self.writeElement_TODO(ti.ident, kind, ti.span, |_self : &mut Self| { 
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
        
        self.writeElement_TODO(ii.ident, kind, ii.span, |_self : &mut Self| { 
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
        
        self.write_function_element(ident, span, fd, &|_self : &mut Self| { 
            walk_fn(_self, fk, fd, b, span);
        });
    }
    
    fn visit_foreign_item(&mut self, foreign_item: &'v ForeignItem) { 
        let kind;
        
        match foreign_item.node {
            ForeignItemKind::Fn(ref _function_declaration, ref _generics) => {
                kind = StructureElementKind::Function;
            }
            ForeignItemKind::Static(ref _typ, _) => {
                kind = StructureElementKind::Var;
            }
        }
        
        self.writeElement_TODO(foreign_item.ident, kind, foreign_item.span, |_self : &mut Self| { 
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
        //panic!("visit_mac disabled by default");
        
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
fn tests_describe_structure() {
    
    use std::rc::Rc;
    use std::cell::RefCell;
    use parse_describe;
    use token_writer::TokenWriter;
    use util::tests::*;

    
    fn test_describe_structure(source : &str, expected : &str) {
        let stringRc = Rc::new(RefCell::new(String::new()));
        
        {
            let (messages, elements) = parse_describe::parse_crate_with_messages(source);
            
            let mut tokenWriter = TokenWriter { out : stringRc.clone() };
            parse_describe::write_parse_analysis_contents(messages, elements, &mut tokenWriter).ok().unwrap();
        }
        
        let expected : &str = &(String::from("MESSAGES {\n}") + 
            (if expected.is_empty() { "" } else { "\n" }) + expected);
        
        let result = unwrap_Rc_RefCell(stringRc);
        let result = result.trim();
        check_equal(result, expected);
    }
    
    test_describe_structure("extern crate xx;", r#"ExternCrate { "xx" { 0:0 0:16 } {} "" {} }"#);
    
    test_describe_structure("const xx : u32 = 1;", r#"Var { "xx" { 0:0 0:19 } {} "u32" {} }"#);
    
    
    test_describe_structure("mod myMod   ;  ", r#"Mod { "myMod" { 0:0 0:13 } {} "" {} }"#);
    test_describe_structure("mod myMod { }", r#"Mod { "myMod" { 0:0 0:13 } {} "" {} }"#);
    test_describe_structure("mod myMod { static xx : u32 = 2; }", 
r#"Mod { "myMod" { 0:0 0:34 } {} "" {}
  Var { "xx" { 0:12 0:32 } {} "u32" {} }
}"#
    );
    
    test_describe_structure("fn xx() { }", r#"Function { "xx" { 0:0 0:11 } {} "()" {} }"#);
    test_describe_structure("fn xx(a : &str) -> u32 { }", 
        r#"Function { "xx" { 0:0 0:26 } {} "(&str) -> u32" {} }"#);
    test_describe_structure("fn xx(blah : Vec<u32>, x : &'v str) -> u32 { }", 
        r#"Function { "xx" { 0:0 0:46 } {} "(Vec<u32>, &'v str) -> u32" {} }"#);
    
    test_describe_structure("type MyType = &u32<asd>;", r#"TypeAlias { "MyType" { 0:0 0:24 } {} "" {} }"#);
    
    test_describe_structure("enum MyEnum { Alpha, Beta, } ", 
r#"Enum { "MyEnum" { 0:0 0:28 } {} "" {}
  EnumVariant { "Alpha" { 0:14 0:19 } {} "" {} }
  EnumVariant { "Beta" { 0:21 0:25 } {} "" {} }
}"#);
    test_describe_structure("enum MyEnum<T, U> { Alpha(T), Beta(U), } ", 
r#"Enum { "MyEnum" { 0:0 0:40 } {} "" {}
  EnumVariant { "Alpha" { 0:20 0:28 } {} "" {} }
  EnumVariant { "Beta" { 0:30 0:37 } {} "" {} }
}"#);
    
    
    test_describe_structure("struct MyStruct ( u32, blah<sdf> ); ", 
r#"Struct { "MyStruct" { 0:0 0:35 } {} "" {} }"#);
    test_describe_structure("struct MyStruct { foo : u32, } ", 
r#"Struct { "MyStruct" { 0:0 0:30 } {} "" {}
  Var { "foo" { 0:18 0:27 } {} "" {} }
}"#);
    
    test_describe_structure("trait MyTrait { } ", r#"Trait { "MyTrait" { 0:0 0:17 } {} "" {} }"#);
    test_describe_structure("trait MyTrait : Foo { fn xxx(); } ", 
r#"Trait { "MyTrait" { 0:0 0:33 } {} "" {}
  Function { "xxx" { 0:22 0:31 } {} "()" {} }
}"#);
    test_describe_structure("trait MyTrait : Foo { type N: fmt::Display; fn xxx(&self); const foo :u32 = 3; } ", 
r#"Trait { "MyTrait" { 0:0 0:80 } {} "" {}
  TypeAlias { "N" { 0:22 0:43 } {} "" {} }
  Function { "xxx" { 0:44 0:58 } {} "(self)" {} }
  Var { "foo" { 0:59 0:78 } {} "" {} }
}"#);
    
    /* FIXME: review
    test_describe_structure("impl MyType { } ", r#"Impl { "MyType" { 0:0 0:15 } {} "" {} }"#);
    test_describe_structure("impl MyTrait for MyType { } ", r#"Impl { "MyType.MyTrait" { 0:0 0:27 } {} "" {} }"#);
    test_describe_structure("impl  MyTrait       { type N= fmt::Display; fn xx(){} const foo :u32 = 3; } ", 
r#"Impl { "MyTrait" { 0:0 0:75 } {} "" {}
  TypeAlias { "N" { 0:22 0:43 } {} "" {} }
  Function { "xx" { 0:44 0:53 } {} "()" {} }
  Var { "foo" { 0:54 0:73 } {} "" {} }
}"#);
    */
    
    
    test_describe_structure("use blah;", r#"Use { "blah" { 0:0 0:9 } {} "" {} }"#);
    test_describe_structure("use blah as foo;", r#"Use { "blah as foo" { 0:0 0:16 } {} "" {} }"#);
    // TODO: this is not printing the global path prefix, seems to be a limitation from libsyntax?
    test_describe_structure("use ::blah::foo as myfoo;", r#"Use { "::blah::foo as myfoo" { 0:0 0:25 } {} "" {} }"#);
    test_describe_structure("use ::blah::foo::*;", r#"Use { "::blah::foo::*" { 0:0 0:19 } {} "" {} }"#);
    test_describe_structure("use blah::foo:: { One as OtherOne, self as Two };", 
        r#"Use { "blah::foo::{ One as OtherOne, self as Two, }" { 0:0 0:49 } {} "" {} }"#);
    
    
    test_describe_structure("my_macro!(asf); ", "");
    
    // test: visit_mac! visit method 
    test_describe_structure("fn foo() { my_macro!(asf); }", r#"Function { "foo" { 0:0 0:28 } {} "()" {} }"#);
    
    test_describe_structure("macro_rules! foo { (x => $e:expr) => (); }", "");
    
    // Test pub extern
    test_describe_structure("pub extern crate my_crate;", 
        r#"ExternCrate { "my_crate" { 0:0 0:26 } {} "" {} }"#
    );
    
    test_describe_structure("extern { fn ext(p : u32); }", 
r#"Mod { "" { 0:0 0:27 } {} "" {}
  Function { "ext" { 0:9 0:25 } {} "" {} }
}"#);
    test_describe_structure("extern { fn ext(p : u32); \n static extVar: u8; }", 
r#"Mod { "" { 0:0 1:21 } {} "" {}
  Function { "ext" { 0:9 0:25 } {} "" {} }
  Var { "extVar" { 1:1 1:19 } {} "" {} }
}"#);
    
    // Test with a lexer error, 
//    test_describe_structure("const xx : u32 = '", r#"Var { "xx" { 1 0 1 19 } {} {} {} }"#);    
}