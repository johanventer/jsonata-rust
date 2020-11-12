use std::fmt;

use crate::Position;

/// An object is represented as a list of (key, value) tuples
pub type Object = Vec<(Node, Node)>;

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
    ArrayConstructor,

    /// An object constructor, e.g. `{ key1: value1, key2: value2 }`.
    ObjectConstructor,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnaryOp::*;
        write!(
            f,
            "{}",
            match self {
                Minus => "-",
                ArrayConstructor => "[",
                ObjectConstructor => "{",
            }
        )
    }
}

/// Types of binary expressions.
#[derive(Debug, Copy, Clone)]
pub enum BinaryOp {
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

    /// Path operator, e.g. `x.y`.
    PathOp,

    /// An array range index, e.g. `[x..y]`.
    Range,

    /// A context variable binding, e.g. `library.loans@$l`.
    ContextBind,

    /// A positional variable binding, e.g. `library.books#$i`.
    PositionalBind,

    /// An filtering predicate, e.g. `phone.number[type="mobile"]`.
    Predicate,

    /// Group by
    GroupBy,

    /// Chained function application, e.g. `$func1 ~> $func2`.
    Apply,

    /// A variable binding, e.g. `$x := 10`.
    Bind,

    /// An array sorting expression, e.g. `account.order.product^(price)`.
    SortOp,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryOp::*;
        write!(
            f,
            "{}",
            match self {
                PathOp => ".",
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
                GroupBy => "{",
                Apply => "~>",
                Bind => ":=",
                SortOp => "^",
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

    /// An index expression.
    Index,

    /// A ternary conditional expression.
    Ternary,

    /// An object transform expression.
    Transform,

    /// A path consisting of multiple steps.
    Path,

    /// A sort consisting of multiple sort terms.
    Sort,

    /// A sort term. The associated value indicates whether it is a descending term.
    SortTerm(bool),
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
                Index => "Index".to_string(),
                Ternary => "Ternary".to_string(),
                Transform => "Transform".to_string(),
                Path => "Path".to_string(),
                Sort => "Sort".to_string(),
                SortTerm(ref v) => format!("SortTerm({})", v),
            }
        )
    }
}

#[derive(Debug)]
pub struct GroupBy {
    pub position: Position,
    pub object: Object,
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

    /// Indicates that this node should not be flattened.
    pub keep_array: bool,

    /// An optional group by expression, represented as an object.
    pub group_by: Option<GroupBy>,

    /// An optional predicate.
    pub predicate: Option<Box<Node>>,

    /// An optional list of evaluation stages, for example this specifies the filtering and
    /// indexing for various expressions.
    pub stages: Option<Vec<Node>>,
    // /// Indicates if this node has a focussed variable binding.
    // pub focus: Option<String>,

    // /// Indicates if this node has an indexed variable binding.
    // pub index: Option<String>,
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
            keep_array: false,
            group_by: None,
            predicate: None,
            stages: None,
            // focus: None,
            // index: None,
            // tuple: false,
        }
    }

    pub fn is_path(&self) -> bool {
        matches!(self.kind, NodeKind::Path)
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let children = self.children.iter().cloned().collect();
        let stages = if let Some(stages) = &self.stages {
            Some(stages.iter().cloned().collect())
        } else {
            None
        };
        let group_by = if let Some(group_by) = &self.group_by {
            Some(GroupBy {
                position: group_by.position,
                object: group_by.object.iter().cloned().collect(),
            })
        } else {
            None
        };

        Self {
            kind: self.kind.clone(),
            position: self.position,
            children,
            predicate: self.predicate.clone(),
            stages,
            group_by,
            keep_array: self.keep_array,
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
