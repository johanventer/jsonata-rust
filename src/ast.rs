use std::fmt;

use crate::tokenizer::Position;

/// An object is represented as a list of (key, value) tuples
pub type Object = Vec<(Node, Node)>;

/// Slots are used for resolving path ancestory.
#[derive(Debug)]
pub struct Slot {
    pub label: String,
    pub level: u32,
    pub index: u32,
}

/// Types of unary expressions.
#[derive(Debug)]
pub enum UnaryOp {
    /// Unary numeric minux, e.g. `-1`.
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
#[derive(Debug, Clone)]
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
#[derive(Debug)]
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
    Parent(Option<Slot>),

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
                Null => "Null".to_owned(),
                Bool(ref v) => format!("Bool({})", v),
                Str(ref v) => format!("Str({})", v),
                Num(ref v) => format!("Num({})", v),
                Name(ref v) => format!("Name({})", v),
                Var(ref v) => format!("Var({})", v),
                Unary(ref v) => format!("Unary({})", v),
                Binary(ref v) => format!("Binary({})", v),
                Wildcard => "Wildcard".to_string(),
                Descendent => "Descendent".to_string(),
                Parent(_) => "Parent".to_string(),
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

    /// An optional group by expression, represented as an object.
    pub group_by: Option<Object>,

    /// An optional list of predicates.
    pub predicates: Option<Vec<Node>>,

    /// An optional list of evaluation stages, for example this specifies the filtering and
    /// indexing for various expressions.
    pub stages: Option<Vec<Node>>,

    /// Indicates if this node has a focussed variable binding.
    pub focus: Option<String>,

    /// Indicates if this node has an indexed variable binding.
    pub index: Option<String>,

    /// Indicates whether the name indicated by this node should be kept as a singleton array.
    pub keep_array: bool,

    /// TODO: I'm not really sure what this indicates, yet, but it is used during evaluation.
    pub tuple: bool,
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
            group_by: None,
            predicates: None,
            stages: None,
            focus: None,
            index: None,
            keep_array: false,
            tuple: false,
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

// ==========================================================================================================

///// This is a poorly named bucket of methods that every node type should implement.
/////
///// Mostly used for error messages and transformation to JSON.
//pub trait NodeMethods {
//    fn get_position(&self) -> usize;
//    fn get_value(&self) -> String;
//    fn to_json(&self) -> JsonValue;
//}

///// An AST node.
/////
///// Each node's associated value contains all of the pertinent AST information required for it.
//#[derive(Debug)]
//pub enum Node {
//    Null(LiteralNode<NullValue, "value">),
//    Boolean(LiteralNode<bool, "value">),
//    Str(LiteralNode<String, "string">),
//    Number(LiteralNode<f64, "number">),
//    Name(LiteralNode<String, "name">),
//    Variable(LiteralNode<String, "variable">),
//    PathSeparator(BinaryNode<".">),
//    Add(BinaryNode<"+">),
//    Subtract(BinaryNode<"-">),
//    Multiply(BinaryNode<"*">),
//    Divide(BinaryNode<"/">),
//    Modulus(BinaryNode<"%">),
//    Equal(BinaryNode<"=">),
//    LessThan(BinaryNode<"<">),
//    GreaterThan(BinaryNode<">">),
//    NotEqual(BinaryNode<"!=">),
//    LessThanEqual(BinaryNode<"<=">),
//    GreaterThanEqual(BinaryNode<">=">),
//    Concat(BinaryNode<"&">),
//    And(BinaryNode<"and">),
//    Or(BinaryNode<"or">),
//    In(BinaryNode<"in">),
//    Chain(BinaryNode<"~>">),
//    Wildcard(MarkerNode<"wildcard">),
//    DescendantWildcard(MarkerNode<"descendent">),
//    ParentOp(MarkerNode<"parent">),
//    FunctionCall(FunctionCallNode<"function">),
//    PartialFunctionCall(FunctionCallNode<"partial">),
//    PartialFunctionArg(MarkerNode<"?">),
//    LambdaFunction(LambdaNode),
//    UnaryMinus(UnaryNode<"-">),
//    Block(ExpressionsNode<"block", "(">),
//    Array(ExpressionsNode<"unary", "[">),
//    Range(BinaryNode<"..">),
//    Assignment(BinaryNode<":=">),
//    OrderBy(OrderByNode),
//    SortTerm(SortTermNode),
//    FocusVariableBind(BinaryNode<"@">),
//    IndexVariableBind(BinaryNode<"#">),
//    Ternary(TernaryNode),
//    Transform(TransformNode),
//    Object(ObjectNode),
//    ObjectGroup(ObjectGroupNode),
//    ArrayPredicate(BinaryNode<"[">),

//    // Nodes created by last-stage AST processing
//    Path(PathNode),
//    Parent(ParentNode),
//    Bind(BindNode),
//    Apply(ApplyNode),
//    Sort(SortNode),
//    // Filter
//    // Index
//}

///// A helper macro to forward calls through to the contained nodes, so we only have one big
///// match branch to update when node types are added, instead of one for every method.
//macro_rules! delegate {
//    ($s:ident, $f:ident) => {
//        match $s {
//            Node::Null(n) => n.$f(),
//            Node::Boolean(n) => n.$f(),
//            Node::Str(n) => n.$f(),
//            Node::Number(n) => n.$f(),
//            Node::Name(n) => n.$f(),
//            Node::Variable(n) => n.$f(),
//            Node::PathSeparator(n) => n.$f(),
//            Node::Add(n) => n.$f(),
//            Node::Subtract(n) => n.$f(),
//            Node::Multiply(n) => n.$f(),
//            Node::Divide(n) => n.$f(),
//            Node::Modulus(n) => n.$f(),
//            Node::Equal(n) => n.$f(),
//            Node::LessThan(n) => n.$f(),
//            Node::GreaterThan(n) => n.$f(),
//            Node::NotEqual(n) => n.$f(),
//            Node::LessThanEqual(n) => n.$f(),
//            Node::GreaterThanEqual(n) => n.$f(),
//            Node::Concat(n) => n.$f(),
//            Node::And(n) => n.$f(),
//            Node::Or(n) => n.$f(),
//            Node::In(n) => n.$f(),
//            Node::Chain(n) => n.$f(),
//            Node::Wildcard(n) => n.$f(),
//            Node::DescendantWildcard(n) => n.$f(),
//            Node::ParentOp(n) => n.$f(),
//            Node::FunctionCall(n) => n.$f(),
//            Node::PartialFunctionCall(n) => n.$f(),
//            Node::PartialFunctionArg(n) => n.$f(),
//            Node::LambdaFunction(n) => n.$f(),
//            Node::UnaryMinus(n) => n.$f(),
//            Node::Block(n) => n.$f(),
//            Node::Range(n) => n.$f(),
//            Node::Array(n) => n.$f(),
//            Node::Assignment(n) => n.$f(),
//            Node::OrderBy(n) => n.$f(),
//            Node::SortTerm(n) => n.$f(),
//            Node::FocusVariableBind(n) => n.$f(),
//            Node::IndexVariableBind(n) => n.$f(),
//            Node::Ternary(n) => n.$f(),
//            Node::Transform(n) => n.$f(),
//            Node::Object(n) => n.$f(),
//            Node::ObjectGroup(n) => n.$f(),
//            Node::ArrayPredicate(n) => n.$f(),
//            Node::Path(n) => n.$f(),
//            Node::Parent(n) => n.$f(),
//            Node::Bind(n) => n.$f(),
//            Node::Apply(n) => n.$f(),
//            Node::Sort(n) => n.$f(),
//        }
//    };
//}

///// Implements the methods that all nodes should respond to by delegating to the associated value
///// of the variants.
//impl NodeMethods for Node {
//    fn get_position(&self) -> usize {
//        delegate!(self, get_position)
//    }

//    fn get_value(&self) -> String {
//        delegate!(self, get_value)
//    }

//    fn to_json(&self) -> JsonValue {
//        delegate!(self, to_json)
//    }
//}

///// A marker struct for a `null` value (in the Javascript sense of `null`).
//#[derive(Debug, Clone)]
//pub struct NullValue {}

//impl fmt::Display for NullValue {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        write!(f, "null")
//    }
//}

//impl From<NullValue> for JsonValue {
//    fn from(_: NullValue) -> Self {
//        JsonValue::Null
//    }
//}

///// A literal node, containing only a literal value.
//#[derive(Debug)]
//pub struct LiteralNode<T, const KIND: &'static str>
//where
//    T: Into<JsonValue> + Clone + fmt::Display,
//{
//    pub position: usize,
//    pub value: T,
//    /// Specifies that this literal node is a path step which should be kept as an array
//    pub keep_array: bool,
//}

//impl<T, const TYPE: &'static str> LiteralNode<T, TYPE>
//where
//    T: Into<JsonValue> + Clone + fmt::Display,
//{
//    pub fn new(position: usize, value: T) -> Self {
//        Self {
//            position,
//            value,
//            keep_array: false,
//        }
//    }
//}

//impl<T, const TYPE: &'static str> NodeMethods for LiteralNode<T, TYPE>
//where
//    T: Into<JsonValue> + Clone + fmt::Display,
//{
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        format!("{}", self.value)
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: TYPE,
//            position: self.position,
//            value: self.value.clone().into(),
//            keepArray: self.keep_array
//        }
//    }
//}

///// A unary node has only a value and an expression. It represents things like unary minus, i.e
///// `-1`.
//#[derive(Debug)]
//pub struct UnaryNode<const VALUE: &'static str> {
//    pub position: usize,
//    pub expression: Box<Node>,
//}

//impl<const VALUE: &'static str> NodeMethods for UnaryNode<VALUE> {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        VALUE.to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "unary",
//            value: VALUE,
//            position: self.position,
//            expression: self.expression.to_json()
//        }
//    }
//}

///// A binary node, with a left hand side and a right hand side.
//#[derive(Debug)]
//pub struct BinaryNode<const VALUE: &'static str> {
//    pub position: usize,
//    pub lhs: Box<Node>,
//    pub rhs: Box<Node>,
//}

//impl<const VALUE: &'static str> NodeMethods for BinaryNode<VALUE> {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        VALUE.to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "binary",
//            value: VALUE,
//            position: self.position,
//            lhs: self.lhs.to_json(),
//            rhs: self.rhs.to_json()
//        }
//    }
//}

///// An marker node is used for nodes that don't have any additional information. This is
///// used for the path navigation operators like `**`, `*` and `%`, as well as partial function
///// arguments.
//#[derive(Debug)]
//pub struct MarkerNode<const TYPE: &'static str> {
//    pub position: usize,
//}

//impl<const TYPE: &'static str> NodeMethods for MarkerNode<TYPE> {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        unreachable!();
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: TYPE,
//            position: self.position,
//        }
//    }
//}

///// A function call, which has a procedure to call and a vector of arguments.
//#[derive(Debug)]
//pub struct FunctionCallNode<const TYPE: &'static str> {
//    pub position: usize,
//    pub procedure: Box<Node>,
//    pub arguments: Vec<Box<Node>>,
//}

//impl<const TYPE: &'static str> NodeMethods for FunctionCallNode<TYPE> {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        self.procedure.get_value()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: TYPE,
//            position: self.position,
//            procedure: self.procedure.to_json(),
//            arguments: self.arguments.iter().map(|arg| arg.to_json()).collect::<Vec<_>>(),
//        }
//    }
//}

///// The definition of a lambda function, including it's arguments and body.
//#[derive(Debug)]
//pub struct LambdaNode {
//    pub position: usize,
//    pub arguments: Vec<Box<Node>>,
//    pub body: Box<Node>,
//}

//impl NodeMethods for LambdaNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "function".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "lambda",
//            position: self.position,
//            arguments: self.arguments.iter().map(|arg| arg.to_json()).collect::<Vec<_>>(),
//            body: self.body.to_json()
//        }
//    }
//}

///// An expressions node contains a vector of expressions, for things like blocks and array
///// definitions.
//#[derive(Debug)]
//pub struct ExpressionsNode<const TYPE: &'static str, const VALUE: &'static str> {
//    pub position: usize,
//    pub expressions: Vec<Box<Node>>,

//    /// Notates that this node is a path contructor, used in Node::Array
//    pub consarray: bool,
//}

//impl<const TYPE: &'static str, const VALUE: &'static str> ExpressionsNode<TYPE, VALUE> {
//    pub fn new(position: usize, expressions: Vec<Box<Node>>) -> Self {
//        Self {
//            position,
//            expressions,
//            consarray: false,
//        }
//    }
//}

//impl<const TYPE: &'static str, const VALUE: &'static str> NodeMethods
//    for ExpressionsNode<TYPE, VALUE>
//{
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        VALUE.to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: TYPE,
//            value: VALUE,
//            position: self.position,
//            expressions: self.expressions.iter().map(|expr| expr.to_json()).collect::<Vec<_>>(),
//            consarray: self.consarray
//        }
//    }
//}

///// The order-by operator, which specifies sorting for arrays by one or more terms.
//#[derive(Debug)]
//pub struct OrderByNode {
//    pub position: usize,
//    pub lhs: Box<Node>,
//    pub rhs: Vec<SortTermNode>,
//}

//impl NodeMethods for OrderByNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "^".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "binary",
//            value: "^",
//            position: self.position,
//            lhs: self.lhs.to_json(),
//            rhs: self.rhs.iter().map(|expr| expr.to_json()).collect::<Vec<_>>()
//        }
//    }
//}

///// A term of the order-by operator.
//#[derive(Debug)]
//pub struct SortTermNode {
//    pub position: usize,
//    pub expression: Box<Node>,
//    pub descending: bool,
//}

//impl NodeMethods for SortTermNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        self.expression.get_value()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            position: self.position,
//            expression: self.expression.to_json(),
//            descending: self.descending
//        }
//    }
//}

///// The ternary condition node, i.e `? :`.
//#[derive(Debug)]
//pub struct TernaryNode {
//    pub position: usize,
//    pub condition: Box<Node>,
//    pub then: Box<Node>,
//    pub els: Option<Box<Node>>,
//}

//impl NodeMethods for TernaryNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "?:".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "condition",
//            position: self.position,
//            condition: self.condition.to_json(),
//            then: self.then.to_json(),
//            else: self.els.as_ref().map_or(JsonValue::Null, |els| els.to_json())
//        }
//    }
//}

///// The object transform node, for the update/delete object transformers.
//#[derive(Debug)]
//pub struct TransformNode {
//    pub position: usize,
//    pub pattern: Box<Node>,
//    pub update: Box<Node>,
//    pub delete: Option<Box<Node>>,
//}

//impl NodeMethods for TransformNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "|".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "transform",
//            position: self.position,
//            pattern: self.pattern.to_json(),
//            update: self.update.to_json(),
//            delete: self.delete.as_ref().map_or(JsonValue::Null, |delete| delete.to_json())
//        }
//    }
//}

///// An object definition.
//#[derive(Debug)]
//pub struct ObjectNode {
//    pub position: usize,
//    pub lhs: Vec<(Box<Node>, Box<Node>)>,
//}

//impl NodeMethods for ObjectNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "{".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "unary",
//            position: self.position,
//            lhs: self.lhs.iter().map(|(name, value)| array![name.to_json(), value.to_json()]).collect::<Vec<_>>()
//        }
//    }
//}

///// Object group by
//#[derive(Debug)]
//pub struct ObjectGroupNode {
//    pub position: usize,
//    pub lhs: Box<Node>,
//    pub rhs: Vec<(Box<Node>, Box<Node>)>,
//}

//impl NodeMethods for ObjectGroupNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "{".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "binary",
//            position: self.position,
//            lhs: self.lhs.to_json(),
//            rhs: self.rhs.iter().map(|(name, value)| array![name.to_json(), value.to_json()]).collect::<Vec<_>>()
//        }
//    }
//}

//#[derive(Debug)]
//pub struct ParentNode {
//    pub position: usize,
//    pub slot: Slot,
//}

//impl NodeMethods for ParentNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        // TODO: This should probably return the slot?
//        "%".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "%",
//            position: self.position,
//            // TODO: slot: self.slot.to_json()
//        }
//    }
//}

///// An object path.
//#[derive(Debug)]
//pub struct PathNode {
//    pub steps: Vec<Box<Node>>,
//    pub seeking_parent: Vec<Slot>,
//    pub keep_singleton_array: bool,
//}

//impl NodeMethods for PathNode {
//    fn get_position(&self) -> usize {
//        unreachable!();
//    }

//    fn get_value(&self) -> String {
//        unreachable!();
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "path",
//            steps: self.steps.iter().map(|step| step.to_json()).collect::<Vec<_>>(),
//            keepSingletonArray: self.keep_singleton_array
//        }
//    }
//}

///// Binding assignment.
//#[derive(Debug)]
//pub struct BindNode {
//    pub position: usize,
//    pub lhs: Box<Node>,
//    pub rhs: Box<Node>,
//}

//impl NodeMethods for BindNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        ":=".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "bind",
//            value: ":=",
//            lhs: self.lhs.to_json(),
//            rhs: self.rhs.to_json()
//        }
//    }
//}

///// Function application.
//#[derive(Debug)]
//pub struct ApplyNode {
//    pub position: usize,
//    pub lhs: Box<Node>,
//    pub rhs: Box<Node>,
//}

//impl NodeMethods for ApplyNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        "~>".to_string()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "apply",
//            value: "~>",
//            lhs: self.lhs.to_json(),
//            rhs: self.rhs.to_json()
//        }
//    }
//}

///// Array sort.
//#[derive(Debug)]
//pub struct SortNode {
//    pub position: usize,
//    pub terms: Vec<SortTermNode>,
//}

//impl NodeMethods for SortNode {
//    fn get_position(&self) -> usize {
//        self.position
//    }

//    fn get_value(&self) -> String {
//        unreachable!()
//    }

//    fn to_json(&self) -> JsonValue {
//        object! {
//            type: "sort",
//            terms: self.terms.iter().map(|t| t.to_json()).collect::<Vec<_>>()
//        }
//    }
//}
