#![feature(or_patterns)]
#![feature(box_syntax)]

mod error;
mod evaluator;
mod functions;
mod jsonata;
mod parser;

pub use jsonata::JsonAta;

use error::JsonAtaError;

pub type JsonAtaResult<T> = std::result::Result<T, Box<dyn JsonAtaError>>;

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
    pub fn advance_x(&mut self, x: usize) {
        self.column += x;
        self.source_pos += x;
    }

    pub fn advance_line(&mut self) {
        self.line += 1;
        self.column = 0;
        self.source_pos += 1;
    }

    pub fn advance_1(&mut self) {
        self.advance_x(1);
    }

    pub fn advance_2(&mut self) {
        self.advance_x(2);
    }
}
