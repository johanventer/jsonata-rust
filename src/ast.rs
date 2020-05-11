use json::{object, JsonValue};

pub trait Node {
    fn to_json(&self) -> JsonValue;
}

pub struct NullNode {
    pub position: usize,
}

impl Node for NullNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "value",
            position: self.position,
            value: JsonValue::Null
        }
    }
}

pub struct BooleanNode {
    pub position: usize,
    pub value: bool,
}

impl Node for BooleanNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "value",
            position: self.position,
            value: JsonValue::from(self.value)
        }
    }
}

pub struct StringNode {
    pub position: usize,
    pub value: String,
}

impl Node for StringNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "string",
            position: self.position,
            value: JsonValue::from(&self.value[..])
        }
    }
}

pub struct NumberNode {
    pub position: usize,
    pub value: f64,
}

impl Node for NumberNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "number",
            position: self.position,
            value: JsonValue::from(self.value)
        }
    }
}

pub struct NameNode {
    pub position: usize,
    pub value: String,
}

impl Node for NameNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "name",
            position: self.position,
            value: JsonValue::from(&self.value[..])
        }
    }
}

pub struct VariableNode {
    pub position: usize,
    pub value: String,
}

impl Node for VariableNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "variable",
            position: self.position,
            value: JsonValue::from(&self.value[..])
        }
    }
}

pub struct AddNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for AddNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "+",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct SubtractNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for SubtractNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "-",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct MultiplyNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for MultiplyNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "*",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct DivideNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for DivideNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "/",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct ModulusNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for ModulusNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "%",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct EqualNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for EqualNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "=",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct LessThanNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for LessThanNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "<",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct GreaterThanNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for GreaterThanNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: ">",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct NotEqualNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for NotEqualNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "!=",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct LessEqualNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for LessEqualNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "<=",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct GreaterEqualNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for GreaterEqualNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: ">=",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct ConcatNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for ConcatNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "&",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct AndNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for AndNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "and",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct OrNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for OrNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "or",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}

pub struct InNode {
    pub position: usize,
    pub lhs: Box<dyn Node>,
    pub rhs: Box<dyn Node>,
}

impl Node for InNode {
    fn to_json(&self) -> JsonValue {
        object! {
            type: "binary",
            value: "in",
            position: self.position,
            lhs: self.lhs.to_json(),
            rhs: self.rhs.to_json()
        }
    }
}
