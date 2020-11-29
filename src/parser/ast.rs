use super::Position;

pub type Object = Vec<(Box<Node>, Box<Node>)>;

#[derive(Debug)]
pub enum UnaryOp {
    Minus(Box<Node>),
    ArrayConstructor(Vec<Box<Node>>),
    ObjectConstructor(Object),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Concat,
    And,
    Or,
    In,
    Path,
    Range,
    ContextBind,
    PositionalBind,
    Predicate,
    Apply,
    Bind,
}

#[derive(Debug)]
pub enum NodeKind {
    Null,
    Bool(bool),
    Str(String),
    Num(f64),
    Name(String),
    Var(String),
    Unary(UnaryOp),
    Binary(BinaryOp, Box<Node>, Box<Node>),
    GroupBy(Box<Node>, Object),
    SortOp(Box<Node>, Vec<Box<Node>>),
    Block(Vec<Box<Node>>),
    Wildcard,
    Descendent,
    Parent,
    Function {
        proc: Box<Node>,
        args: Vec<Box<Node>>,
        is_partial: bool,
    },
    PartialArg,
    Lambda {
        args: Vec<Box<Node>>,
        body: Box<Node>,
    },
    Ternary {
        cond: Box<Node>,
        truthy: Box<Node>,
        falsy: Option<Box<Node>>,
    },
    Transform {
        pattern: Box<Node>,
        update: Box<Node>,
        delete: Option<Box<Node>>,
    },
    SortTerm(Box<Node>, bool),
    Path(Vec<Box<Node>>),
    Sort(Vec<Box<Node>>),
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub position: Position,

    pub keep_array: bool,
    pub cons_array: bool,
    pub keep_singleton_array: bool,

    /// An optional group by expression, represented as an object.
    pub group_by: Option<Object>,

    /// An optional list of predicates.
    pub predicates: Option<Vec<Box<Node>>>,

    /// An optional list of evaluation stages, for example this specifies the filtering and
    /// indexing for various expressions.
    pub stages: Option<Vec<Box<Node>>>,
}

impl Node {
    pub(crate) fn new(kind: NodeKind, position: Position) -> Self {
        Self {
            kind,
            position,
            keep_array: false,
            cons_array: false,
            keep_singleton_array: false,
            group_by: None,
            predicates: None,
            stages: None,
        }
    }

    #[inline]
    pub(crate) fn is_path(&self) -> bool {
        matches!(self.kind, NodeKind::Path{..})
    }

    #[inline]
    pub(crate) fn path_len(&self) -> usize {
        match self.kind {
            NodeKind::Path(ref steps) => steps.len(),
            _ => panic!("Not a path"),
        }
    }

    #[inline]
    pub(crate) fn new_path(position: Position, steps: Vec<Box<Node>>) -> Self {
        Node::new(NodeKind::Path(steps), position)
    }

    #[inline]
    pub(crate) fn path_steps(&mut self) -> &mut Vec<Box<Node>> {
        match self.kind {
            NodeKind::Path(ref mut steps) => steps,
            _ => panic!("Not a path"),
        }
    }

    #[inline]
    pub(crate) fn push_step(&mut self, step: Box<Node>) {
        match self.kind {
            NodeKind::Path(ref mut steps) => steps.push(step),
            _ => panic!("Not a path"),
        }
    }

    #[inline]
    pub(crate) fn append_steps(&mut self, new_steps: &mut Vec<Box<Node>>) {
        match self.kind {
            NodeKind::Path(ref mut steps) => steps.append(new_steps),
            _ => panic!("Not a path"),
        }
    }

    #[inline]
    pub(crate) fn take_path_steps(self) -> Vec<Box<Node>> {
        match self.kind {
            NodeKind::Path(steps) => steps,
            _ => panic!("Not a path"),
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Minus(..) => "-",
                ArrayConstructor(..) => "[",
                ObjectConstructor(_) => "{",
            }
        )
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Path { .. } => ".",
                Add => "+",
                Subtract => "-",
                Multiply => "*",
                Divide => "/",
                Modulus => "%",
                Equal => "=",
                NotEqual => "!=",
                LessThan => "<",
                LessThanEqual => "<=",
                GreaterThan => ">",
                GreaterThanEqual => ">=",
                Concat => "&",
                And => "and",
                Or => "or",
                In => "in",
                Range => "..",
                ContextBind => "@",
                PositionalBind => "#",
                Predicate => "[",
                Apply => "~>",
                Bind => ":=",
            }
        )
    }
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NodeKind::*;
        write!(
            f,
            "{}",
            match self {
                Null => "null".to_owned(),
                Bool(ref v) => v.to_string(),
                Str(ref v) => v.to_string(),
                Num(ref v) => v.to_string(),
                Name(ref v) => v.to_string(),
                Var(ref v) => v.to_string(),
                Unary(ref v) => v.to_string(),
                Binary(ref v, ..) => v.to_string(),
                Wildcard => "**".to_string(),
                Descendent => "*".to_string(),
                Parent => "%".to_string(),
                Function { .. } => "Function".to_string(),
                PartialArg => "PartialArg".to_string(),
                Lambda { .. } => "Lambda".to_string(),
                Block(..) => "Block".to_string(),
                Ternary { .. } => "Ternary".to_string(),
                Transform { .. } => "Transform".to_string(),
                Path { .. } => "Path".to_string(),
                Sort(..) => "Sort".to_string(),
                SortTerm(_, ref v) => format!("SortTerm({})", v),
                GroupBy(..) => "{".to_string(),
                SortOp(..) => "^".to_string(),
            }
        )
    }
}
