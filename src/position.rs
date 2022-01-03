#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub source_pos: usize,
}

impl Position {
    pub fn advance(&mut self, x: usize) {
        self.column += x;
        self.source_pos += x;
    }

    pub fn advance1(&mut self) {
        self.advance(1);
    }

    pub fn advance2(&mut self) {
        self.advance(2);
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}:{}]", self.line, self.column)
    }
}
