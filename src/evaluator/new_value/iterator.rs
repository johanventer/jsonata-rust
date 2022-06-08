use std::iter::Iterator;

use super::{Value, ValueKind, ValuePtr};

pub struct MemberIterator<'a, 'arena> {
    value: &'a ValueKind<'arena>,
    front: usize,
    back: usize,
    back_done: bool,
}

impl<'a, 'arena> MemberIterator<'a, 'arena> {
    pub fn new(value: &'a ValueKind<'arena>) -> Self {
        let len = value.len();
        Self {
            value,
            front: 0,
            back: len.saturating_sub(1),
            back_done: false,
        }
    }
}

impl<'a, 'arena> Iterator for MemberIterator<'a, 'arena> {
    type Item = Value<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front < self.value.len() {
            let result = match *self.value {
                ValueKind::Array(ref array, ..) => array.get(self.front).copied(),
                ValueKind::Range(ref range) => range.nth(self.front),
                _ => unreachable!(),
            };
            self.front += 1;
            result
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for MemberIterator<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_done {
            return None;
        }

        let result = match *self.value {
            ValueKind::Array(ref array, _) => array.get(self.back).copied(),
            ValueKind::Range(ref range) => range.nth(self.back),
            _ => unreachable!(),
        };

        if self.back == 0 {
            self.back_done = true;
            return result;
        }

        self.back -= 1;
        result
    }
}

pub struct EntryIterator<'a, 'arena> {
    iter: hashbrown::hash_map::Iter<'a, &'arena str, Value<'arena>>,
}

impl<'a, 'arena> EntryIterator<'a, 'arena> {
    pub fn new(value: &'a ValueKind<'arena>) -> Self {
        match value {
            ValueKind::Object(hash) => Self { iter: hash.iter() },
            _ => unreachable!(),
        }
    }
}

impl<'a, 'arena> Iterator for EntryIterator<'a, 'arena> {
    type Item = (&'arena str, Value<'arena>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, value)| (*key, *value))
    }
}

#[cfg(test)]
mod tests {
    use super::super::ValueArena;
    use super::*;

    #[test]
    fn member_forward() {
        let arena = ValueArena::new();
        let range = arena.range(1, 5);
        let kind = range.borrow();
        let mut iter = MemberIterator::new(&*kind);

        assert_eq!(*iter.next().unwrap().borrow(), 1_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 2_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 3_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 4_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 5_isize);
        assert!(iter.next().is_none());
    }

    #[test]
    fn member_backward() {
        let arena = ValueArena::new();
        let range = arena.range(1, 5);
        let kind = range.borrow();
        let mut iter = MemberIterator::new(&*kind);
        assert_eq!(*iter.next_back().unwrap().borrow(), 5_isize);
        assert_eq!(*iter.next_back().unwrap().borrow(), 4_isize);
        assert_eq!(*iter.next_back().unwrap().borrow(), 3_isize);
        assert_eq!(*iter.next_back().unwrap().borrow(), 2_isize);
        assert_eq!(*iter.next_back().unwrap().borrow(), 1_isize);
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn member_reverse() {
        let arena = ValueArena::new();
        let range = arena.range(1, 5);
        let kind = range.borrow();
        let mut iter = MemberIterator::new(&*kind).rev();
        assert_eq!(*iter.next().unwrap().borrow(), 5_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 4_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 3_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 2_isize);
        assert_eq!(*iter.next().unwrap().borrow(), 1_isize);
        assert!(iter.next().is_none());
    }

    #[test]
    fn entries() {
        let arena = ValueArena::new();
        let object = arena.object();
        object.borrow_mut().insert("Hello", arena.string("World"));
        let kind = object.borrow();
        let mut iter = EntryIterator::new(&*kind);
        let (k, v) = iter.next().unwrap();
        assert_eq!(k, "Hello");
        assert_eq!(*v.borrow(), "World");
        assert!(iter.next().is_none());
    }
}
