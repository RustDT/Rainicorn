
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