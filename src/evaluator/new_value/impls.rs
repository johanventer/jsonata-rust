use super::ValueKind;

impl<'arena> PartialEq<ValueKind<'arena>> for ValueKind<'_> {
    fn eq(&self, other: &ValueKind<'arena>) -> bool {
        match (self, other) {
            (ValueKind::Undefined, ValueKind::Undefined) => true,
            (ValueKind::Null, ValueKind::Null) => true,
            (ValueKind::Number(l), ValueKind::Number(r)) => *l == *r,
            (ValueKind::Bool(l), ValueKind::Bool(r)) => *l == *r,
            (ValueKind::String(l), ValueKind::String(r)) => *l == *r,
            // (ValueKind::Array(l, ..), ValueKind::Array(r, ..)) => *l == *r,
            // (ValueKind::Object(l), ValueKind::Object(r)) => *l == *r,
            (ValueKind::Range(l), ValueKind::Range(r)) => *l == *r,
            _ => false,
        }
    }
}

impl PartialEq<bool> for ValueKind<'_> {
    fn eq(&self, other: &bool) -> bool {
        matches!(self, ValueKind::Bool(ref b) if *b == *other)
    }
}

impl PartialEq<usize> for ValueKind<'_> {
    fn eq(&self, other: &usize) -> bool {
        matches!(self, ValueKind::Number(..) if self.as_usize() == *other)
    }
}

impl PartialEq<isize> for ValueKind<'_> {
    fn eq(&self, other: &isize) -> bool {
        matches!(self, ValueKind::Number(..) if self.as_isize() == *other)
    }
}

impl PartialEq<&str> for ValueKind<'_> {
    fn eq(&self, other: &&str) -> bool {
        matches!(self, ValueKind::String(ref s) if s == *other)
    }
}

impl std::fmt::Debug for ValueKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Number(n) => n.fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Array(a, _) => a.fmt(f),
            Self::Object(o) => o.fmt(f),
            // Self::Lambda { .. } => write!(f, "<lambda>"),
            // Self::NativeFn { .. } => write!(f, "<nativefn>"),
            Self::Transformer { .. } => write!(f, "<transformer>"),
            Self::Range(r) => write!(f, "<range({},{})>", r.start(), r.end()),
        }
    }
}

impl std::string::ToString for ValueKind<'_> {
    fn to_string(&self) -> String {
        format!("{:#?}", self)
    }
}
