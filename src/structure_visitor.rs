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

use crate::source_model::*;
use crate::util::core::*;

use crate::rustc_ast::ast::*;
use crate::rustc_span::source_map::{SourceMap, Span};
use crate::rustc_span::symbol::{Symbol, Ident};
use crate::rustc_ast::visit::*;

use std::ops::Fn;

pub struct StructureVisitor<'ps> {
    pub codemap: &'ps SourceMap,
    pub parentIsStruct: bool,
    pub parentIsUnion: bool,
    pub elements: Vec<StructureElement>,
}

impl<'ps> StructureVisitor<'ps> {
    pub fn new(codemap: &'ps SourceMap) -> StructureVisitor<'ps> {
        StructureVisitor {
            codemap: codemap,
            parentIsStruct: false,
            parentIsUnion: false,
            elements: vec![],
        }
    }

    fn visit_mac(&mut self, mac: &'ps MacCall) {
        walk_mac(self, mac)
    }

    pub fn write_element_do<FN>(&mut self, ident: &str, kind: StructureElementKind, sourcerange: SourceRange, type_desc: String, walkFn: FN) -> Void
    where
        FN: Fn(&mut Self),
    {
        let mut siblings = vec![];
        std::mem::swap(&mut self.elements, &mut siblings);

        walkFn(self); // self.elements now has children

        std::mem::swap(&mut self.elements, &mut siblings);
        let children = siblings;

        let element = StructureElement {
            name: String::from(ident),
            kind: kind,
            sourcerange: sourcerange,
            type_desc: type_desc,
            children: children,
        };

        self.elements.push(element);
        Ok(())
    }

    pub fn write_element_handled<FN>(&mut self, ident: &str, kind: StructureElementKind, sourceRange: SourceRange, type_desc: String, walkFn: FN)
    where
        FN: Fn(&mut Self),
    {
        use std::io::Write;

        match self.write_element_do(ident, kind, sourceRange, type_desc, walkFn) {
            Ok(ok) => ok,
            Err(error) => {
                std::io::stderr().write_fmt(format_args!("Error writing element: {}", error)).ok();
            }
        }
    }

    pub fn write_element_TODO<FN>(&mut self, ident: Ident, kind: StructureElementKind, span: Span, walkFn: FN)
    where
        FN: Fn(&mut Self),
    {
        self.write_element(ident, kind, span, "".to_string(), walkFn);
    }

    pub fn write_element<FN>(&mut self, ident: Ident, kind: StructureElementKind, span: Span, type_desc: String, walkFn: FN)
    where
        FN: Fn(&mut Self),
    {
        self.write_element_handled(&*ident.name.as_str(), kind, SourceRange::new(self.codemap, span), type_desc, walkFn)
    }

    /* -----------------  ----------------- */

    fn write_ItemUse(&mut self, tree: &UseTree) {
        use std::ops::Index;
        use crate::rustc_ast::ast;
        use crate::rustc_ast_pretty::pprust;

        let kind = StructureElementKind::Use;
        let mut useSpec = String::new();

        fn writePath(outString: &mut String, path: &ast::Path) {
            outString.push_str(&pprust::path_to_string(path));
        }

        match tree.kind {
            UseTreeKind::Simple(ref ident, _ , _) => {
                writePath(&mut useSpec, &tree.prefix);

                let path: &ast::Path = &tree.prefix;
                if path.segments.len() == 0 {
                    return;
                }
                let lastSegment = path.segments.index(path.segments.len() - 1);
                    if let Some(ident) = ident {
                        if &lastSegment.ident != ident {
                        useSpec.push_str(&" as ");
                        useSpec.push_str(&*ident.name.as_str());
                    }
                }
            }
            UseTreeKind::Glob => {
                useSpec.push_str(&pprust::path_to_string(&tree.prefix));
                useSpec.push_str(&"::*");
            }
            UseTreeKind::Nested(ref trees) => {
                writePath(&mut useSpec, &tree.prefix);

                useSpec.push_str("::{ ");
                for treenode in trees {
                    self.write_ItemUse(&treenode.0);
                }
                useSpec.push_str("}");
            }
        }

        self.write_element_handled(&useSpec, kind, SourceRange::new(self.codemap, tree.span), "".to_string(), &|_: &mut Self| {})
    }

    fn get_type_desc_from_fndecl(&mut self, fd: &FnDecl) -> String {
        let mut type_desc = "".to_string();

        type_desc.push('(');
        let mut needs_sep = false;

        for arg in &fd.inputs {
            let pat_span = arg.pat.span;

            if needs_sep {
                type_desc.push_str(", ");
            }
            needs_sep = true;

            if let Ok(snippet) = self.codemap.span_to_snippet(arg.ty.span) {
                if !snippet.is_empty() {
                    type_desc.push_str(&snippet);
                    continue;
                }
            }

            if let Ok(snippet) = self.codemap.span_to_snippet(pat_span) {
                if snippet.ends_with("self") {
                    type_desc.push_str(&snippet);
                    continue;
                }
            }
        }
        type_desc.push_str(")");

        if let FnRetTy::Ty(ref _ret) = fd.output {
            if let Ok(ret_snippet) = self.codemap.span_to_snippet(fd.output.span()) {
                type_desc.push_str(" -> ");
                type_desc.push_str(&ret_snippet);
            }
        }

        type_desc
    }

    fn write_function_element(&mut self, ident: Ident, span: Span, fd: &FnDecl, walkFn: &dyn Fn(&mut Self)) {
        let type_desc = self.get_type_desc_from_fndecl(&fd);

        self.write_element(ident, StructureElementKind::Function, span, type_desc, walkFn);
    }
}

impl<'v> Visitor<'v> for StructureVisitor<'v> {
    fn visit_name(&mut self, _span: Span, _name: Symbol) {
        // Nothing to do.
    }
    fn visit_ident(&mut self, ident: Ident) {
        walk_ident(self, ident);
    }

    fn visit_item(&mut self, item: &'v Item) {
        let kind;
        let mut type_desc = "".to_string();

        let noop_walkFn = &|_self: &mut Self| {};

        let walkFn: &dyn Fn(&mut Self) = &|_self: &mut Self| {
            walk_item(_self, item);
        };

        match item.kind {
            ItemKind::GlobalAsm(ref asm) => {
            	self.visit_inline_asm(asm);
            	return;
            }
            ItemKind::TraitAlias(ref _generics, ref _bounds) => {
                kind = StructureElementKind::TraitAlias;
            }
            ItemKind::ExternCrate(_opt_name) => {
                kind = StructureElementKind::ExternCrate;
            }
            ItemKind::Use(ref vp) => {
                self.write_ItemUse(vp);
                return;
            }
            ItemKind::Static(ref typ, _, ref _expr) | ItemKind::Const(_, ref typ, ref _expr) => {
                if let Ok(snippet) = self.codemap.span_to_snippet(typ.span) {
                    type_desc.push_str(&snippet);
                }
                self.write_element(item.ident, StructureElementKind::Var, item.span, type_desc, noop_walkFn);
                return;
            }
            ItemKind::Fn(ref fn1) => {
            	self.visit_fn(FnKind::Fn(FnCtxt::Free, item.ident, &fn1.sig, &item.vis, &fn1.generics, fn1.body.as_ref().map(|p| &**p)), item.span, item.id);
                return;
            }
            ItemKind::Mod(_, _) => {
                kind = StructureElementKind::Mod;
            }
            ItemKind::ForeignMod(ref _foreign_module) => {
                kind = StructureElementKind::Mod;
            }
            ItemKind::TyAlias(_) => {
                kind = StructureElementKind::TypeAlias;
            }
            ItemKind::Enum(ref _enum_definition, ref _type_parameters) => {
                kind = StructureElementKind::Enum;
            }
            ItemKind::Impl(_) => {
                kind = StructureElementKind::Impl;
            }
            ItemKind::Struct(ref _struct_definition, ref _generics) => {
                // Go straight in
                self.parentIsStruct = true;
                walk_item(self, item);
                return;
            }
            ItemKind::Union(ref _struct_definition, ref _generics) => {
                self.parentIsUnion = true;
                walk_item(self, item);
                return;
            }
            ItemKind::Trait(_) => {
                kind = StructureElementKind::Trait;
            }
            ItemKind::MacCall(ref mac) => {
                self.visit_mac(mac);
                return;
            }
            ItemKind::MacroDef(ref macro_def) => {
                self.visit_mac_def(macro_def, item.id);
                return;
            }
        }

        self.write_element(item.ident, kind, item.span, type_desc, walkFn);
    }

    fn visit_enum_def(&mut self, enum_def: &'v EnumDef, generics: &'v Generics, nodeid: NodeId, _span: Span) {
        // This element is covered by an item definition
        walk_enum_def(self, enum_def, generics, nodeid)
    }

    fn visit_variant(&mut self, v: &'v Variant) {
        // This element is covered by an enum_def call
        walk_variant(self, v);
    }

    fn visit_variant_data(&mut self, s: &'v VariantData) {
        let mut kind = StructureElementKind::EnumVariant;
        if self.parentIsStruct {
            kind = StructureElementKind::Struct;
            self.parentIsStruct = false;
        }
        if self.parentIsUnion {
            kind = StructureElementKind::Union;
            self.parentIsUnion = false;
        }
        match s {
            VariantData::Struct(fields, _) | VariantData::Tuple(fields, _) => {
                for field in fields {
                    self.write_element_TODO(field.ident.unwrap_or(Ident::empty()), kind, field.span, |_self: &mut Self| {
                        walk_struct_def(_self, s);
                    });
                }
            },
            VariantData::Unit(_) => return,
        };
    }

    /* ----------------- Function ----------------- */

    fn visit_fn(&mut self, fk: FnKind<'v>, span: Span, _nodeid: NodeId) {
        let ident: Ident;

        match fk {
            FnKind::Fn(_, ident1, ref _MethodSig, _option, _b, _) => {
                ident = ident1;
            }
            FnKind::Closure(_, _) => {
                return;
            }
        };

        self.write_function_element(ident, span, fk.decl(), &|_self: &mut Self| {
            walk_fn(_self, fk, span);
        });
    }

    fn visit_foreign_item(&mut self, foreign_item: &'v ForeignItem) {
        let kind;

        match foreign_item.kind {
            ForeignItemKind::Fn(_) => {
                kind = StructureElementKind::Function;
            }
            ForeignItemKind::Static(ref _typ, _, _) => {
                kind = StructureElementKind::Var;
            }
            ForeignItemKind::TyAlias(_) => {
                kind = StructureElementKind::TypeAlias;
            }
            ForeignItemKind::MacCall(_) => {
                kind = StructureElementKind::MacroDef;
            }
        }

        self.write_element_TODO(foreign_item.ident, kind, foreign_item.span, |_self: &mut Self| {
            walk_foreign_item(_self, foreign_item);
        });
    }

    fn visit_trait_ref(&mut self, t: &'v TraitRef) {
        walk_trait_ref(self, t)
    }
    fn visit_poly_trait_ref(&mut self, t: &'v PolyTraitRef, m: &'v TraitBoundModifier) {
        walk_poly_trait_ref(self, t, m)
    }

    fn visit_lifetime(&mut self, lifetime: &'v Lifetime) {
        walk_lifetime(self, lifetime)
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
    fn visit_expr(&mut self, _ex: &Expr) {
        // Comment, no need to visit node insinde expressions
        //walk_expr(self, ex)
    }
    fn visit_expr_post(&mut self, _ex: &Expr) {}
    fn visit_ty(&mut self, t: &'v Ty) {
        walk_ty(self, t)
    }
    fn visit_generics(&mut self, g: &'v Generics) {
        walk_generics(self, g)
    }
    fn visit_path(&mut self, path: &'v Path, _id: NodeId) {
        walk_path(self, path)
    }
    fn visit_path_segment(&mut self, path_span: Span, path_segment: &'v PathSegment) {
        walk_path_segment(self, path_span, path_segment)
    }
    fn visit_attribute(&mut self, _attr: &Attribute) {}
    fn visit_vis(&mut self, vis: &'v Visibility) {
        walk_vis(self, vis)
    }
}

#[test]
fn tests_describe_structure() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    use crate::parse_describe;
    use crate::token_writer::TokenWriter;
    use crate::util::tests::*;

    fn test_describe_structure(source: &str, expected: &str) {
        let stringRc = Rc::new(RefCell::new(String::new()));

        {
            let (messages, elements) = parse_describe::parse_crate_with_messages(source);

            let mut tokenWriter = TokenWriter { out: stringRc.clone() };
            parse_describe::write_parse_analysis_contents(messages, elements, &mut tokenWriter).ok().unwrap();
        }

        let expected: &str = &(String::from("MESSAGES {\n}") + (if expected.is_empty() { "" } else { "\n" }) + expected);

        let result = unwrap_Rc_RefCell(stringRc);
        let result = result.trim();
        check_equal(result, expected);
    }

    test_describe_structure("extern crate xx;", r#"ExternCrate { "xx" { 0:0 0:16 } {} "" {} }"#);

    test_describe_structure("const xx : u32 = 1;", r#"Var { "xx" { 0:0 0:19 } {} "u32" {} }"#);

    test_describe_structure("mod myMod   ;  ", r#"Mod { "myMod" { 0:0 0:13 } {} "" {} }"#);
    test_describe_structure("mod myMod { }", r#"Mod { "myMod" { 0:0 0:13 } {} "" {} }"#);
    test_describe_structure(
        "mod myMod { static xx : u32 = 2; }",
        r#"Mod { "myMod" { 0:0 0:34 } {} "" {}
  Var { "xx" { 0:12 0:32 } {} "u32" {} }
}"#,
    );

    test_describe_structure("fn xx() { }", r#"Function { "xx" { 0:0 0:11 } {} "()" {} }"#);
    test_describe_structure("fn xx(a : &str) -> u32 { }", r#"Function { "xx" { 0:0 0:26 } {} "(&str) -> u32" {} }"#);
    test_describe_structure("fn xx(blah : Vec<u32>, x : &'v str) -> u32 { }", r#"Function { "xx" { 0:0 0:46 } {} "(Vec<u32>, &'v str) -> u32" {} }"#);
    // Test "deceiving" case
    test_describe_structure("fn xx(my_self : &str) -> u32 { }", r#"Function { "xx" { 0:0 0:32 } {} "(&str) -> u32" {} }"#);

    test_describe_structure("type MyType = &u32<asd>;", r#"TypeAlias { "MyType" { 0:0 0:24 } {} "" {} }"#);

    test_describe_structure(
        "enum MyEnum { Alpha, Beta, } ",
        r#"Enum { "MyEnum" { 0:0 0:28 } {} "" {}
  EnumVariant { "Alpha" { 0:14 0:19 } {} "" {} }
  EnumVariant { "Beta" { 0:21 0:25 } {} "" {} }
}"#,
    );
    test_describe_structure(
        "enum MyEnum<T, U> { Alpha(T), Beta(U), } ",
        r#"Enum { "MyEnum" { 0:0 0:40 } {} "" {}
  EnumVariant { "Alpha" { 0:20 0:28 } {} "" {} }
  EnumVariant { "Beta" { 0:30 0:37 } {} "" {} }
}"#,
    );

    test_describe_structure("struct MyStruct ( u32, blah<sdf> ); ", r#"Struct { "MyStruct" { 0:0 0:35 } {} "" {} }"#);
    test_describe_structure(
        "struct MyStruct { foo : u32, } ",
        r#"Struct { "MyStruct" { 0:0 0:30 } {} "" {}
  Var { "foo" { 0:18 0:27 } {} "" {} }
}"#,
    );
    test_describe_structure(
        "union MyUnion { foo : u32, } ",
        r#"Union { "MyUnion" { 0:0 0:28 } {} "" {}
  Var { "foo" { 0:16 0:25 } {} "" {} }
}"#,
    );

    test_describe_structure("trait MyTrait { } ", r#"Trait { "MyTrait" { 0:0 0:17 } {} "" {} }"#);
    test_describe_structure(
        "trait MyTrait : Foo { fn xxx(); } ",
        r#"Trait { "MyTrait" { 0:0 0:33 } {} "" {}
  Function { "xxx" { 0:22 0:31 } {} "()" {} }
}"#,
    );
    test_describe_structure(
        "trait MyTrait : Foo { type N: fmt::Display; fn xxx(&self); const foo :u32 = 3; } ",
        r#"Trait { "MyTrait" { 0:0 0:80 } {} "" {}
  TypeAlias { "N" { 0:22 0:43 } {} "" {} }
  Function { "xxx" { 0:44 0:58 } {} "(&self)" {} }
  Var { "foo" { 0:59 0:78 } {} "" {} }
}"#,
    );

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
    test_describe_structure("use blah::foo:: { One as OtherOne, self as Two };", r#"Use { "blah::foo::{ One as OtherOne, self as Two, }" { 0:0 0:49 } {} "" {} }"#);

    test_describe_structure("my_macro!(asf); ", "");

    // test: visit_mac! visit method
    test_describe_structure("fn foo() { my_macro!(asf); }", r#"Function { "foo" { 0:0 0:28 } {} "()" {} }"#);

    test_describe_structure("macro_rules! foo { (x => $e:expr) => (); }", r#""#);
    // TODO: macro definitions, unfortunately can't get that info easily from syntax_syntex
    //    test_describe_structure("macro_rules! five_times { ($x:expr) => (5 * $x); }",
    //        r#"Macro { "foo" { 0:0 0:28 } {} "" {} }"#);

    // Test pub extern
    test_describe_structure("pub extern crate my_crate;", r#"ExternCrate { "my_crate" { 0:0 0:26 } {} "" {} }"#);

    test_describe_structure(
        "extern { fn ext(p : u32); }",
        r#"Mod { "" { 0:0 0:27 } {} "" {}
  Function { "ext" { 0:9 0:25 } {} "" {} }
}"#,
    );
    test_describe_structure(
        "extern { fn ext(p : u32); \n static extVar: u8; }",
        r#"Mod { "" { 0:0 1:21 } {} "" {}
  Function { "ext" { 0:9 0:25 } {} "" {} }
  Var { "extVar" { 1:1 1:19 } {} "" {} }
}"#,
    );

    // Test with a lexer error,
    //    test_describe_structure("const xx : u32 = '", r#"Var { "xx" { 1 0 1 19 } {} {} {} }"#);
}
