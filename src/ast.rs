use json::{array, object, JsonValue};
use std::fmt;

/// This is a poorly named bucket of methods that every node type should implement.
///
/// Mostly used for error messages and transformation to JSON.
pub trait NodeMethods {
    fn get_position(&self) -> usize;
    fn get_value(&self) -> String;
    fn to_json(&self) -> JsonValue;
}

/// An AST node.
///
/// Each node's associated value contains all of the pertinent AST information required for it.
#[derive(Debug, PartialEq)]
pub enum Node {
    Null(LiteralNode<NullValue, "value">),
    Boolean(LiteralNode<bool, "value">),
    String(LiteralNode<String, "string">),
    Number(LiteralNode<f64, "number">),
    Name(LiteralNode<String, "name">),
    Variable(LiteralNode<String, "variable">),
    PathSeparator(BinaryNode<".">),
    Add(BinaryNode<"+">),
    Subtract(BinaryNode<"-">),
    Multiply(BinaryNode<"*">),
    Divide(BinaryNode<"/">),
    Modulus(BinaryNode<"%">),
    Equal(BinaryNode<"=">),
    LessThan(BinaryNode<"<">),
    GreaterThan(BinaryNode<">">),
    NotEqual(BinaryNode<"!=">),
    LessThanEqual(BinaryNode<"<=">),
    GreaterThanEqual(BinaryNode<">=">),
    Concat(BinaryNode<"&">),
    And(BinaryNode<"and">),
    Or(BinaryNode<"or">),
    In(BinaryNode<"in">),
    Chain(BinaryNode<"~>">),
    Wildcard(EmptyNode<"*">),
    DescendantWildcard(EmptyNode<"**">),
    Parent(EmptyNode<"%">),
    FunctionCall(FunctionCallNode<"function">),
    PartialFunctionCall(FunctionCallNode<"partial">),
    PartialFunctionArg(EmptyNode<"?">),
    LambdaFunction(LambdaNode),
    UnaryMinus(UnaryNode<"-">),
    Block(ExpressionsNode<"block">),
    Array(ExpressionsNode<"unary">),
    Range(BinaryNode<"..">),
    Assignment(BinaryNode<":=">),
    OrderBy(OrderByNode),
    OrderByTerm(OrderByTermNode),
    FocusVariableBind(BinaryNode<"@">),
    IndexVariableBind(BinaryNode<"#">),
    Ternary(TernaryNode),
    Transform(TransformNode),
    ObjectPrefix(ObjectPrefixNode),
    ObjectInfix(ObjectInfixNode),
    ArrayPredicate(BinaryNode<"[">),
}

/// A helper macro to forward calls through to the contained nodes, so we only have one big
/// match branch to update when node types are added, instead of one for every method.
macro_rules! delegate {
    ($s:ident, $f:ident) => {
        match $s {
            Node::Null(n) => n.$f(),
            Node::Boolean(n) => n.$f(),
            Node::String(n) => n.$f(),
            Node::Number(n) => n.$f(),
            Node::Name(n) => n.$f(),
            Node::Variable(n) => n.$f(),
            Node::PathSeparator(n) => n.$f(),
            Node::Add(n) => n.$f(),
            Node::Subtract(n) => n.$f(),
            Node::Multiply(n) => n.$f(),
            Node::Divide(n) => n.$f(),
            Node::Modulus(n) => n.$f(),
            Node::Equal(n) => n.$f(),
            Node::LessThan(n) => n.$f(),
            Node::GreaterThan(n) => n.$f(),
            Node::NotEqual(n) => n.$f(),
            Node::LessThanEqual(n) => n.$f(),
            Node::GreaterThanEqual(n) => n.$f(),
            Node::Concat(n) => n.$f(),
            Node::And(n) => n.$f(),
            Node::Or(n) => n.$f(),
            Node::In(n) => n.$f(),
            Node::Chain(n) => n.$f(),
            Node::Wildcard(n) => n.$f(),
            Node::DescendantWildcard(n) => n.$f(),
            Node::Parent(n) => n.$f(),
            Node::FunctionCall(n) => n.$f(),
            Node::PartialFunctionCall(n) => n.$f(),
            Node::PartialFunctionArg(n) => n.$f(),
            Node::LambdaFunction(n) => n.$f(),
            Node::UnaryMinus(n) => n.$f(),
            Node::Block(n) => n.$f(),
            Node::Range(n) => n.$f(),
            Node::Array(n) => n.$f(),
            Node::Assignment(n) => n.$f(),
            Node::OrderBy(n) => n.$f(),
            Node::OrderByTerm(n) => n.$f(),
            Node::FocusVariableBind(n) => n.$f(),
            Node::IndexVariableBind(n) => n.$f(),
            Node::Ternary(n) => n.$f(),
            Node::Transform(n) => n.$f(),
            Node::ObjectPrefix(n) => n.$f(),
            Node::ObjectInfix(n) => n.$f(),
            Node::ArrayPredicate(n) => n.$f(),
        }
    };
}

/// Implements the methods that all nodes should respond to by delegating to the associated value
/// of the variants.
impl NodeMethods for Node {
    fn get_position(&self) -> usize {
        delegate!(self, get_position)
    }

    fn get_value(&self) -> String {
        delegate!(self, get_value)
    }

    fn to_json(&self) -> JsonValue {
        delegate!(self, to_json)
    }
}

/// A marker struct for a `null` value (in the Javascript sense of `null`).
#[derive(Debug, Clone, PartialEq)]
pub struct NullValue {}

impl fmt::Display for NullValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

impl From<NullValue> for JsonValue {
    fn from(_: NullValue) -> Self {
        JsonValue::Null
    }
}

/// A literal node, containing only a literal value.
#[derive(Debug, PartialEq)]
pub struct LiteralNode<T, const KIND: &'static str>
where
    T: Into<JsonValue> + Clone + fmt::Display,
{
    pub position: usize,
    pub value: T,
}

impl<T, const TYPE: &'static str> NodeMethods for LiteralNode<T, TYPE>
where
    T: Into<JsonValue> + Clone + fmt::Display,
{
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        format!("{}", self.value)
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            value: self.value.clone().into()
        }
    }
}

/// A unary node has only a value and an expression. It represents things like unary minus, i.e
/// `-1`.
#[derive(Debug, PartialEq)]
pub struct UnaryNode<const VALUE: &'static str> {
    pub position: usize,
    pub expression: Box<Node>,
}

impl<const VALUE: &'static str> NodeMethods for UnaryNode<VALUE> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        VALUE.to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "unary",
            value: VALUE,
            position: self.position,
            expression: self.expression.to_json()
        }
    }
}

/// A binary node, with a left hand side and a right hand side.
#[derive(Debug, PartialEq)]
pub struct BinaryNode<const VALUE: &'static str> {
    pub position: usize,
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
}

impl<const VALUE: &'static str> NodeMethods for BinaryNode<VALUE> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        VALUE.to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: VALUE,
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

/// An empty node is used for nodes that don't have any additional information. Mostly this is
/// useful for the path navigation operators like `**`, `*` and `%`.
#[derive(Debug, PartialEq)]
pub struct EmptyNode<const TYPE: &'static str> {
    pub position: usize,
}

impl<const TYPE: &'static str> NodeMethods for EmptyNode<TYPE> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        TYPE.to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
        }
    }
}

/// A function call, which has a procedure to call and a vector of arguments.
#[derive(Debug, PartialEq)]
pub struct FunctionCallNode<const TYPE: &'static str> {
    pub position: usize,
    pub procedure: Box<Node>,
    pub arguments: Vec<Node>,
}

impl<const TYPE: &'static str> NodeMethods for FunctionCallNode<TYPE> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        self.procedure.get_value()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            procedure: self.procedure.to_json(),
            arguments: self.arguments.iter().map(|arg| arg.to_json()).collect::<Vec<_>>(),
        }
    }
}

/// The definition of a lambda function, including it's arguments and body.
#[derive(Debug, PartialEq)]
pub struct LambdaNode {
    pub position: usize,
    pub procedure: Box<Node>,
    pub arguments: Vec<Node>,
    pub body: Box<Node>,
}

impl NodeMethods for LambdaNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        self.procedure.get_value()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "lambda",
            position: self.position,
            procedure: self.procedure.to_json(),
            arguments: self.arguments.iter().map(|arg| arg.to_json()).collect::<Vec<_>>(),
            body: self.body.to_json()
        }
    }
}

/// An expressions node contains a vector of expressions, for things like blocks and array
/// definitions.
#[derive(Debug, PartialEq)]
pub struct ExpressionsNode<const TYPE: &'static str> {
    pub position: usize,
    pub expressions: Vec<Node>,
}

impl<const TYPE: &'static str> NodeMethods for ExpressionsNode<TYPE> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        TYPE.to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            expressions: self.expressions.iter().map(|expr| expr.to_json()).collect::<Vec<_>>(),
        }
    }
}

/// The order-by operator, which specifies sorting for arrays by one or more terms.
#[derive(Debug, PartialEq)]
pub struct OrderByNode {
    pub position: usize,
    pub lhs: Box<Node>,
    pub rhs: Vec<Node>,
}

impl NodeMethods for OrderByNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        "^".to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "^",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.iter().map(|expr| expr.to_json()).collect::<Vec<_>>()
        }
    }
}

/// A term of the order-by operator.
#[derive(Debug, PartialEq)]
pub struct OrderByTermNode {
    pub position: usize,
    pub expression: Box<Node>,
    pub descending: bool,
}

impl NodeMethods for OrderByTermNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        self.expression.get_value()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            position: self.position,
            expression: self.expression.to_json(),
            descending: self.descending
        }
    }
}

/// The ternary condition node, i.e `? :`.
#[derive(Debug, PartialEq)]
pub struct TernaryNode {
    pub position: usize,
    pub condition: Box<Node>,
    pub then: Box<Node>,
    pub els: Option<Box<Node>>,
}

impl NodeMethods for TernaryNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        "?:".to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "condition",
            position: self.position,
            condition: self.condition.to_json(),
            then: self.then.to_json(),
            else: self.els.as_ref().map_or(JsonValue::Null, |els| els.to_json())
        }
    }
}

/// The object transform node, for the update/delete object transformers.
#[derive(Debug, PartialEq)]
pub struct TransformNode {
    pub position: usize,
    pub pattern: Box<Node>,
    pub update: Box<Node>,
    pub delete: Option<Box<Node>>,
}

impl NodeMethods for TransformNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        "|".to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "transform",
            position: self.position,
            pattern: self.pattern.to_json(),
            update: self.update.to_json(),
            delete: self.delete.as_ref().map_or(JsonValue::Null, |delete| delete.to_json())
        }
    }
}

/// The prefix variant of an object definition.
#[derive(Debug, PartialEq)]
pub struct ObjectPrefixNode {
    pub position: usize,
    pub lhs: Vec<(Box<Node>, Box<Node>)>,
}

impl NodeMethods for ObjectPrefixNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        "{".to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "unary",
            position: self.position,
            lhs: self.lhs.iter().map(|(name, value)| array![name.to_json(), value.to_json()]).collect::<Vec<_>>()
        }
    }
}

/// The infix variant of an object definition.
#[derive(Debug, PartialEq)]
pub struct ObjectInfixNode {
    pub position: usize,
    pub lhs: Box<Node>,
    pub rhs: Vec<(Box<Node>, Box<Node>)>,
}

impl NodeMethods for ObjectInfixNode {
    fn get_position(&self) -> usize {
        self.position
    }

    fn get_value(&self) -> String {
        "{".to_string()
    }

    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.iter().map(|(name, value)| array![name.to_json(), value.to_json()]).collect::<Vec<_>>()
        }
    }
}

