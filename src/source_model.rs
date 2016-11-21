// Copyright 2016 Bruno Medeiros
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

use ::syntex_syntax::codemap:: { Span, Loc, CodeMap, CharPos};


#[derive(Debug, Clone, Copy)]
pub struct LineColumnPosition {
    /// The (1-based) line number
    pub line: usize,
    /// The (0-based) column offset
    pub col: CharPos,
    
}

#[derive(Debug, Clone, Copy)]
pub struct SourceRange {
    pub start_pos : LineColumnPosition,
    pub end_pos : LineColumnPosition,
}

impl SourceRange {
    pub fn new(codemap : &CodeMap, span : Span) -> SourceRange {
        let startLoc = codemap.lookup_char_pos(span.lo);
        let endLoc = codemap.lookup_char_pos(span.hi);
        
        SourceRange::fromLoc(startLoc, endLoc)
    }
    
    pub fn fromLoc(startLoc : Loc, endLoc : Loc) -> SourceRange {
        SourceRange{ 
            start_pos : LineColumnPosition{ line: startLoc.line, col : startLoc.col }, 
            end_pos : LineColumnPosition{ line: endLoc.line, col : endLoc.col },
        }
    }
    
}

pub fn sourceRange(start_line : usize, start_col : usize, end_line : usize, end_col : usize) -> SourceRange {
    SourceRange { 
        start_pos : LineColumnPosition { line : start_line, col : CharPos(start_col) },
        end_pos : LineColumnPosition { line : end_line, col : CharPos(end_col) },
    }
}


/* -----------------  ----------------- */

//use ::util::core::*;

pub enum Severity {
    INFO,
    WARNING,
    ERROR,
}

impl Severity {
    pub fn to_string(&self) -> &'static str {
        match *self {
            Severity::ERROR => "ERROR",
            Severity::WARNING => "WARNING",
            Severity::INFO => "INFO",
        }
    }
}


pub struct SourceMessage {
    pub severity : Severity,
    pub sourcerange : Option<SourceRange>,
    pub message : String,
}

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


impl StructureElementKind {
    pub fn to_String(&self) -> &'static str {
        match *self {
            StructureElementKind::Var => "Var",
            StructureElementKind::Function => "Function",
            StructureElementKind::Struct => "Struct",
            StructureElementKind::Impl => "Impl",
            StructureElementKind::Trait => "Trait",
            StructureElementKind::Enum => "Enum",
            StructureElementKind::EnumVariant => "EnumVariant",
            StructureElementKind::ExternCrate => "ExternCrate",
            StructureElementKind::Mod => "Mod",
            StructureElementKind::Use => "Use",
            StructureElementKind::TypeAlias => "TypeAlias",
        }
    }
}

pub struct StructureElement {
    pub name: String,
    pub kind: StructureElementKind,
    pub sourcerange: SourceRange,
    
    pub type_desc: String,
    pub children: Vec<StructureElement>,
}