pub mod ast;
mod parser;
mod process;
mod symbol;
mod tokenizer;

pub(crate) use parser::parse;

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub source_pos: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            source_pos: 0,
        }
    }
}

impl Position {
    pub(super) fn advance(&mut self, x: usize) {
        self.column += x;
        self.source_pos += x;
    }

    pub(super) fn advance1(&mut self) {
        self.advance(1);
    }

    pub(super) fn advance2(&mut self) {
        self.advance(2);
    }
}
