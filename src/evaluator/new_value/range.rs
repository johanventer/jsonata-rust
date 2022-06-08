use super::{Value, ValueArena};

#[derive(Debug, Clone)]
pub struct Range<'arena> {
    arena: &'arena ValueArena,
    start: isize,
    end: isize,
}

impl<'arena> Range<'arena> {
    pub fn new(arena: &'arena ValueArena, start: isize, end: isize) -> Self {
        if end < start {
            panic!("Tried to contruct a range with negative length");
        }
        Self { arena, start, end }
    }

    pub fn start(&self) -> isize {
        self.start
    }

    pub fn end(&self) -> isize {
        self.end
    }

    pub fn len(&self) -> usize {
        (self.end - self.start + 1) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn nth(&self, index: usize) -> Option<Value<'arena>> {
        if index < self.len() {
            Some(self.arena.number((self.start + index as isize) as f64))
        } else {
            None
        }
    }
}

impl PartialEq<Range<'_>> for Range<'_> {
    fn eq(&self, other: &Range) -> bool {
        self.start == other.start && self.end == other.end
    }
}

#[cfg(test)]
mod tests {
    use super::super::ValueArena;
    use super::*;

    #[test]
    fn len() {
        let arena = ValueArena::new();
        let range = Range::new(&arena, 1, 5);
        assert_eq!(range.start(), 1);
        assert_eq!(range.end(), 5);
        assert_eq!(range.len(), 5);
        assert!(!range.is_empty());
    }

    #[test]
    fn nth() {
        let arena = ValueArena::new();
        let range = Range::new(&arena, 1, 5);
        assert_eq!(*range.nth(0).unwrap().borrow(), 1_isize);
        assert_eq!(*range.nth(1).unwrap().borrow(), 2_isize);
        assert_eq!(*range.nth(2).unwrap().borrow(), 3_isize);
        assert_eq!(*range.nth(3).unwrap().borrow(), 4_isize);
        assert_eq!(*range.nth(4).unwrap().borrow(), 5_isize);
        assert!(range.nth(5).is_none());
    }

    #[test]
    fn eq() {
        let arena = ValueArena::new();
        let range1 = Range::new(&arena, 1, 5);
        let range2 = Range::new(&arena, 1, 5);
        assert_eq!(range1, range2);
    }

    #[test]
    fn ne() {
        let arena = ValueArena::new();
        let range1 = Range::new(&arena, 2, 5);
        let range2 = Range::new(&arena, 1, 5);
        assert_ne!(range1, range2);
    }

    #[test]
    fn negative() {
        let arena = ValueArena::new();
        let range = Range::new(&arena, -10, -5);
        assert_eq!(*range.nth(0).unwrap().borrow(), -10_isize);
        assert_eq!(*range.nth(1).unwrap().borrow(), -9_isize);
        assert_eq!(*range.nth(2).unwrap().borrow(), -8_isize);
        assert_eq!(*range.nth(3).unwrap().borrow(), -7_isize);
        assert_eq!(*range.nth(4).unwrap().borrow(), -6_isize);
        assert_eq!(*range.nth(5).unwrap().borrow(), -5_isize);
        assert!(range.nth(6).is_none());
    }
}
