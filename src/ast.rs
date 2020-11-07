use std::fmt;

use crate::tokenizer::Position;

// /// An object is represented as a list of (key, value) tuples
// pub type Object = Vec<(Node, Node)>;

// /// Slots are used for resolving path ancestory.
// #[derive(Debug)]
// pub struct Slot {
//     pub label: String,
//     pub level: u32,
//     pub index: u32,
// }

/// Types of unary expressions.
#[derive(Debug, Copy, Clone)]
pub enum UnaryOp {
    /// Unary numeric minus, e.g. `-1`.
    Minus,

    /// Array constructor, e.g. `[1, 2, 3]`.
    Array,

    /// An object constructor, e.g. `{ key1: value1, key2: value2 }`.
    Object,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Minus => "-",
                Array => "[",
                Object => "{",
            }
        )
    }
}

/// Types of binary expressions.
#[derive(Debug, Copy, Clone)]
pub enum BinaryOp {
    /// Path operator, e.g. `x.y`.
    Path,

    /// Numeric addition, e.g. `x + 10`.
    Add,

    /// Numeric subtraction, e.g. `x - 10`.
    Subtract,

    /// Numeric multiplication, e.g. `x * 10`.
    Multiply,

    /// Numeric division, e.g. `x / 10`.
    Divide,

    /// Numeric modulus, e.g. `x % 10`.
    Modulus,

    /// Equality, e.g `x = y`.
    Equal,

    /// Inequality, e.g. `x != y`.
    NotEqual,

    /// Less than comparison, e.g. `x < y`.
    LessThan,

    /// Great than comparison, e.g. `x > y`.
    GreaterThan,

    /// Less than or equal comparison, e.g. `x <= y`.
    LessThanEqual,

    /// Greater than or equal comparison, e.g. `x >= y`.
    GreaterThanEqual,

    /// String concatenation, e.g. `"x" & "y"`.
    Concat,

    /// Boolean and, e.g. `x and y`.
    And,

    /// Boolean or, e.g. `x or y`.
    Or,

    /// Array containment, e.g. `1 in [1, 2 3]`.
    In,

    /// An array range index, e.g. `[x..y]`.
    Range,

    /// A context variable binding, e.g. `library.loans@$l`.
    ContextBind,

    /// A positional variable binding, e.g. `library.books#$i`.
    PositionalBind,

    /// An array filtering predicate, e.g. `phone.number[type="mobile"]`.
    ArrayPredicate,

    /// Chained function application, e.g. `$func1 ~> $func2`.
    Apply,

    /// A variable binding, e.g. `$x := 10`.
    Bind,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Path => ".",
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
                ArrayPredicate => "[",
                Apply => "~>",
                Bind => ":=",
            }
        )
    }
}

/// Types of AST nodes.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// Literal null value, e.g. `null`.
    Null,

    /// Literal boolean value, e.g. `true` or `false`.
    Bool(bool),

    /// Literal string value, e.g. `"hello"`.
    Str(String),

    /// Literal number value, e.g. `1` or `1.23` or `1e-10`.
    Num(f64),

    /// Name expression, e.g. `product`.
    Name(String),

    /// $ variable expression, e.g. `$x`.
    Var(String),

    /// Unary expression.
    Unary(UnaryOp),

    /// Binary expression.
    Binary(BinaryOp),

    /// Wildcard path navigation, e.g. `address.*`.
    Wildcard,

    /// Descendent path navigation, e.g. `**.postcode`.
    Descendent,

    /// Parent operator expression, e.g. `%.order_id`.
    Parent,

    /// Function call. The associated value indicates whether it is a partial application or not.
    Function(bool),

    /// Partial function call argument, e.g. `$func(?)`.
    PartialArg,

    /// Lambda function definition, e.g. `function($x) { $x + 1 }`.
    Lambda,

    /// Block consisting of multiple expressions, e.g. `($x + 1; $x - 1)`.
    Block,

    /// An array sorting expression, e.g. `account.order.product^(price)`.
    Sort,

    /// A sort term. The associated value indicates whether it is a descending term.
    SortTerm(bool),

    /// A filtering expression.
    Filter,

    /// An index expression.
    Index,

    /// A ternary conditional expression.
    Ternary,

    /// An object transform expression.
    Transform,

    /// A path consisting of multiple steps.
    Path,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NodeKind::*;
        write!(
            f,
            "{}",
            match self {
                Null => "null".to_owned(),
                Bool(ref v) => format!("{}", v),
                Str(ref v) => format!("{}", v),
                Num(ref v) => format!("{}", v),
                Name(ref v) => format!("{}", v),
                Var(ref v) => format!("{}", v),
                Unary(ref v) => format!("Unary({})", v),
                Binary(ref v) => format!("Binary({})", v),
                Wildcard => "Wildcard".to_string(),
                Descendent => "Descendent".to_string(),
                Parent => "Parent".to_string(),
                Function(ref v) => format!("Function({})", v),
                PartialArg => "PartialArg".to_string(),
                Lambda => "Lambda".to_string(),
                Block => "Block".to_string(),
                Sort => "Sort".to_string(),
                SortTerm(ref v) => format!("SortTerm({})", v),
                Filter => "Filter".to_string(),
                Index => "Index".to_string(),
                Ternary => "Ternary".to_string(),
                Transform => "Transform".to_string(),
                Path => "Path".to_string(),
            }
        )
    }
}

/// A node in the parsed AST.
#[derive(Debug)]
pub struct Node {
    /// The kind of the node.
    pub kind: NodeKind,

    /// The position in the input source expression.
    pub position: Position,

    /// A general list of child nodes, could represent lhs/rhs, update/transform/delete,
    /// condition/then/else, procedure/arguments etc.
    pub children: Vec<Node>,
    // /// An optional group by expression, represented as an object.
    // pub group_by: Option<Object>,

    // /// An optional list of predicates.
    // pub predicates: Option<Vec<Node>>,

    // /// An optional list of evaluation stages, for example this specifies the filtering and
    // /// indexing for various expressions.
    // pub stages: Option<Vec<Node>>,

    // /// Indicates if this node has a focussed variable binding.
    // pub focus: Option<String>,

    // /// Indicates if this node has an indexed variable binding.
    // pub index: Option<String>,

    // /// Indicates whether the name indicated by this node should be kept as a singleton array.
    // pub keep_array: bool,

    // /// TODO: I'm not really sure what this indicates, yet, but it is used during evaluation.
    // pub tuple: bool,
}

impl Node {
    pub fn new(kind: NodeKind, position: Position) -> Self {
        Self::new_with_children(kind, position, Vec::new())
    }

    pub fn new_with_child(kind: NodeKind, position: Position, child: Node) -> Self {
        Self::new_with_children(kind, position, vec![child])
    }

    pub fn new_with_children(kind: NodeKind, position: Position, children: Vec<Node>) -> Self {
        Self {
            kind,
            position,
            children,
            // group_by: None,
            // predicates: None,
            // stages: None,
            // focus: None,
            // index: None,
            // keep_array: false,
            // tuple: false,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let mut cloned_children = Vec::<Node>::new();
        cloned_children.reserve_exact(self.children.len());
        for child in &self.children {
            cloned_children.push(child.clone());
        }

        Self {
            kind: self.kind.clone(),
            position: self.position,
            children: cloned_children,
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.kind)?;
        if !self.children.is_empty() {
            for child in &self.children {
                writeln!(f, "  {}", child)?;
            }
        }
        Ok(())
    }
}
