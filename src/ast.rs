use json::{object, JsonValue};

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}

pub enum Node {
    Null(LiteralNode<NullValue, "value">),
    Boolean(LiteralNode<bool, "value">),
    String(LiteralNode<String, "string">),
    Number(LiteralNode<f64, "number">),
    Name(LiteralNode<String, "name">),
    Variable(LiteralNode<String, "value">),
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
    Function(FunctionNode<"function">),
    PartialFunction(FunctionNode<"partial">),
    PartialFunctionArg(BasicNode<"?">),
    UnaryMinus(UnaryNode<"-">),
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
            Function(v) => v.to_json(),
            PartialFunction(v) => v.to_json(),
            PartialFunctionArg(v) => v.to_json(),
            UnaryMinus(v) => v.to_json(),
        }
    }
}

/// Placeholder for a null value
#[derive(Clone)]
pub struct NullValue {}

impl From<NullValue> for JsonValue {
    fn from(null: NullValue) -> Self {
        JsonValue::Null
    }
}

pub struct LiteralNode<T, const kind: &'static str>
where
    T: Into<JsonValue> + Clone,
{
    pub position: usize,
    pub value: T,
}

impl<T, const kind: &'static str> ToJson for LiteralNode<T, kind>
where
    T: Into<JsonValue> + Clone,
{
    fn to_json(&self) -> JsonValue {
        object! {
            type: kind,
            position: self.position,
            value: self.value.clone().into()
        }
    }
}

pub struct UnaryNode<const value: &'static str> {
    pub position: usize,
    pub expression: Box<Node>,
}

impl<const value: &'static str> ToJson for UnaryNode<value> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "unary",
            value,
            position: self.position,
            expression: self.expression.to_json(),
        }
    }
}

pub struct BinaryNode<const value: &'static str> {
    pub position: usize,
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
}

impl<const value: &'static str> ToJson for BinaryNode<value> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value,
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct BasicNode<const kind: &'static str> {
    pub position: usize,
}

impl<const kind: &'static str> ToJson for BasicNode<kind> {
    fn to_json(&self) -> JsonValue {
        object! {
            type: kind,
            position: self.position,
        }
    }
}

pub struct FunctionNode<const kind: &'static str> {
    pub position: usize,
    pub procedure: Box<Node>,
    pub arguments: Vec<Box<Node>>,
}

impl<const kind: &'static str> ToJson for FunctionNode<kind> {
    fn to_json(&self) -> JsonValue {
        let mut obj = object! {
            type: kind,
            position: self.position,
            procedure: self.procedure.to_json(),
        };
        obj.insert(
            "arguments",
            self.arguments
                .iter()
                .map(|arg| arg.to_json())
                .collect::<Vec<_>>(),
        );
        obj
    }
}
