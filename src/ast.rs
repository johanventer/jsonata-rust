use json::{object, JsonValue};

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}

pub trait Position {
    fn get_position(&self) -> usize;
}

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
    Subtract(BinaryNode<"%">),
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
    Wildcard(BasicNode<"*">),
    DescendantWildcard(BasicNode<"**">),
    Parent(BasicNode<"%">),
    FunctionCall(FunctionCallNode<"function">),
    PartialFunctionCall(FunctionCallNode<"partial">),
    PartialFunctionArg(BasicNode<"?">),
    LambdaFunction(LambdaNode),
    UnaryMinus(UnaryNode<"-">),
    Block(ExpressionsNode<"block">),
    Array(ExpressionsNode<"unary">),
    Range(BinaryNode<"..">),
    Assignment(BinaryNode<":=">),
}

// TODO(johan): There must be an easier way to do this, a macro perhaps?
impl Position for Node {
    fn get_position(&self) -> usize {
        use Node::*;
        match self {
            Null(v) => v.position,
            Boolean(v) => v.position,
            String(v) => v.position,
            Number(v) => v.position,
            Name(v) => v.position,
            Variable(v) => v.position,
            PathSeparator(v) => v.position,
            Add(v) => v.position,
            Subtract(v) => v.position,
            Multiply(v) => v.position,
            Divide(v) => v.position,
            Modulus(v) => v.position,
            Equal(v) => v.position,
            LessThan(v) => v.position,
            GreaterThan(v) => v.position,
            NotEqual(v) => v.position,
            LessThanEqual(v) => v.position,
            GreaterThanEqual(v) => v.position,
            Concat(v) => v.position,
            And(v) => v.position,
            Or(v) => v.position,
            In(v) => v.position,
            Chain(v) => v.position,
            Wildcard(v) => v.position,
            DescendantWildcard(v) => v.position,
            Parent(v) => v.position,
            FunctionCall(v) => v.position,
            PartialFunctionCall(v) => v.position,
            PartialFunctionArg(v) => v.position,
            LambdaFunction(v) => v.position,
            UnaryMinus(v) => v.position,
            Block(v) => v.position,
            Range(v) => v.position,
            Array(v) => v.position,
            Assignment(v) => v.position,
        }
    }
}

// TODO(johan): There must be an easier way to do this, a macro perhaps?
impl ToJson for Node {
    fn to_json(&self) -> JsonValue {
        use Node::*;
        match self {
            Null(v) => v.to_json(),
            Boolean(v) => v.to_json(),
            String(v) => v.to_json(),
            Number(v) => v.to_json(),
            Name(v) => v.to_json(),
            Variable(v) => v.to_json(),
            PathSeparator(v) => v.to_json(),
            Add(v) => v.to_json(),
            Subtract(v) => v.to_json(),
            Multiply(v) => v.to_json(),
            Divide(v) => v.to_json(),
            Modulus(v) => v.to_json(),
            Equal(v) => v.to_json(),
            LessThan(v) => v.to_json(),
            GreaterThan(v) => v.to_json(),
            NotEqual(v) => v.to_json(),
            LessThanEqual(v) => v.to_json(),
            GreaterThanEqual(v) => v.to_json(),
            Concat(v) => v.to_json(),
            And(v) => v.to_json(),
            Or(v) => v.to_json(),
            In(v) => v.to_json(),
            Chain(v) => v.to_json(),
            Wildcard(v) => v.to_json(),
            DescendantWildcard(v) => v.to_json(),
            Parent(v) => v.to_json(),
            FunctionCall(v) => v.to_json(),
            PartialFunctionCall(v) => v.to_json(),
            PartialFunctionArg(v) => v.to_json(),
            LambdaFunction(v) => v.to_json(),
            UnaryMinus(v) => v.to_json(),
            Block(v) => v.to_json(),
            Range(v) => v.to_json(),
            Array(v) => v.to_json(),
            Assignment(v) => v.to_json(),
        }
    }
}

/// Placeholder for a null value
#[derive(Debug, Clone, PartialEq)]
pub struct NullValue {}

impl From<NullValue> for JsonValue {
    fn from(_: NullValue) -> Self {
        JsonValue::Null
    }
}

#[derive(Debug, PartialEq)]
pub struct LiteralNode<T, const KIND: &'static str>
where
    T: Into<JsonValue> + Clone,
{
    pub position: usize,
    pub value: T,
}

impl<T, const TYPE: &'static str> ToJson for LiteralNode<T, TYPE>
where
    T: Into<JsonValue> + Clone,
{
    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            value: self.value.clone().into()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UnaryNode<const VALUE: &'static str> {
    pub position: usize,
    pub expression: Box<Node>,
}

impl<const VALUE: &'static str> ToJson for UnaryNode<VALUE> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "unary",
            value: VALUE,
            position: self.position,
            expression: self.expression.to_json()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BinaryNode<const VALUE: &'static str> {
    pub position: usize,
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
}

impl<const VALUE: &'static str> ToJson for BinaryNode<VALUE> {
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

#[derive(Debug, PartialEq)]
pub struct BasicNode<const TYPE: &'static str> {
    pub position: usize,
}

impl<const TYPE: &'static str> ToJson for BasicNode<TYPE> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCallNode<const TYPE: &'static str> {
    pub position: usize,
    pub procedure: Box<Node>,
    pub arguments: Vec<Node>,
}

impl<const TYPE: &'static str> ToJson for FunctionCallNode<TYPE> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            procedure: self.procedure.to_json(),
            arguments: self.arguments.iter().map(|arg| arg.to_json()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LambdaNode {
    pub position: usize,
    pub procedure: Box<Node>,
    pub arguments: Vec<Node>,
    pub body: Box<Node>,
}

impl ToJson for LambdaNode {
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

#[derive(Debug, PartialEq)]
pub struct ExpressionsNode<const TYPE: &'static str> {
    pub position: usize,
    pub expressions: Vec<Node>,
}

impl<const TYPE: &'static str> ToJson for ExpressionsNode<TYPE> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: TYPE,
            position: self.position,
            expressions: self.expressions.iter().map(|expr| expr.to_json()).collect::<Vec<_>>(),
        }
    }
}
