use json::{array, JsonValue};
use std::ops::Index;
use std::slice::Iter;

use crate::ast::*;
use crate::error::*;
use crate::frame::{Binding, Frame};
use crate::functions::*;
use crate::JsonAtaResult;

#[derive(Clone, Debug)]
pub enum Input {
    Undefined,
    Value(JsonValue),
    Sequence(Vec<Input>, bool),
}

impl Input {
    pub fn new_seq(keep_array: bool) -> Self {
        Input::Sequence(Vec::new(), keep_array)
    }

    pub fn seq_from(value: &JsonValue) -> Self {
        if value.is_array() {
            Input::Sequence(
                value.members().cloned().map(|v| Input::Value(v)).collect(),
                false,
            )
        } else {
            Input::Sequence(vec![Input::Value(value.clone())], false)
        }
    }

    pub fn iter(&self) -> Iter<'_, Input> {
        match self {
            Input::Sequence(ref seq, ..) => seq.iter(),
            _ => panic!("Only Input::Sequence can be iterated over"),
        }
    }

    pub fn push(&mut self, result: Input) {
        match self {
            Input::Sequence(ref mut seq, ..) => seq.push(result),
            _ => panic!("Only Input::Sequence can be pushed to"),
        }
    }

    pub fn len(&mut self) -> usize {
        match self {
            Input::Sequence(ref mut seq, ..) => seq.len(),
            _ => panic!("Only Input::Sequence has a length"),
        }
    }

    pub fn as_value(&self) -> &JsonValue {
        match self {
            Input::Value(ref value) => value,
            _ => panic!("not an Input::Value"),
        }
    }

    pub fn as_value_mut(&mut self) -> &mut JsonValue {
        match self {
            Input::Value(ref mut value) => value,
            _ => panic!("not an Input::Value"),
        }
    }

    pub fn as_seq_mut(&mut self) -> &mut Vec<Input> {
        match self {
            Input::Sequence(ref mut seq, ..) => seq,
            _ => panic!("not an Input::Sequence"),
        }
    }

    pub fn is_undefined(&self) -> bool {
        match self {
            Input::Undefined => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            Input::Value(..) => true,
            _ => false,
        }
    }

    pub fn is_sequence(&self) -> bool {
        match self {
            Input::Sequence(..) => true,
            _ => false,
        }
    }
}

impl From<Option<JsonValue>> for Input {
    fn from(value: Option<JsonValue>) -> Self {
        match value {
            None => Input::Undefined,
            Some(value) => Input::Value(value),
        }
    }
}

impl Index<usize> for Input {
    type Output = Input;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Input::Sequence(ref seq, ..) => &seq[index],
            _ => panic!("Only Input::Sequence can be indexed"),
        }
    }
}

pub fn evaluate(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    let mut result = match &node.kind {
        NodeKind::Null => Input::Value(JsonValue::Null),
        NodeKind::Bool(ref value) => Input::Value(json::from(*value)),
        NodeKind::Str(ref value) => Input::Value(json::from(value.clone())),
        NodeKind::Num(ref value) => Input::Value(json::from(*value)),
        NodeKind::Name(_) => evaluate_name(node, input)?,
        NodeKind::Unary(_) => evaluate_unary_op(node, input, frame)?,
        NodeKind::Binary(_) => evaluate_binary_op(node, input, frame)?,
        NodeKind::Block => evaluate_block(node, input, frame)?,
        NodeKind::Ternary => evaluate_ternary(node, input, frame)?,
        NodeKind::Var(ref name) => evaluate_variable(name, frame)?,
        NodeKind::Path => evaluate_path(node, input, frame)?,
        _ => unimplemented!("TODO: node kind not yet supported: {}", node.kind),
    };

    // TODO: Predicate and grouping (jsonata.js:127)

    if result.is_sequence() {
        if result.len() == 0 {
            Ok(Input::Undefined)
        } else if result.len() == 1 {
            Ok(result.as_seq_mut().swap_remove(0))
        } else {
            Ok(result)
        }
    } else {
        Ok(result)
    }
}

fn evaluate_path(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    Ok(match input {
        Input::Undefined => Input::Undefined,
        Input::Sequence(..) => panic!("`input` was a Input::Sequence, which is unexpected"),
        Input::Value(input) => {
            let mut input_seq = Input::seq_from(input);

            // TODO: Tuple, singleton array, group expressions (jsonata.js:164)

            let mut result = Input::Undefined;

            for (step_index, step) in node.children.iter().enumerate() {
                result = evaluate_step(step, &input_seq, frame, step_index == input.len() - 1)?;

                match result {
                    Input::Undefined => break,
                    Input::Value(..) => {
                        unreachable!("`evaluate_step` should always return a sequence")
                    }
                    Input::Sequence(ref seq, ..) => {
                        if seq.is_empty() {
                            break;
                        } else {
                            input_seq = result.clone();
                        }
                    }
                }
            }

            result
        }
    })
}

fn evaluate_step(
    node: &Node,
    input_seq: &Input,
    frame: &mut Frame,
    last_step: bool,
) -> JsonAtaResult<Input> {
    // TODO: Sorting (jsonata.js:253)

    let mut result_seq = Input::new_seq(false);

    for input in input_seq.iter() {
        let result = evaluate(node, input, frame)?;

        // TODO: Filtering (jsonata.js:267)

        match result {
            Input::Undefined => (),
            _ => result_seq.push(result),
        }
    }

    return if last_step
        && result_seq.len() == 1
        && result_seq[0].is_value()
        && result_seq[0].as_value().is_array()
    {
        Ok(Input::Value(result_seq[0].as_value().clone()))
    } else {
        // Flatten the result
        let mut flat_result = Input::new_seq(false);
        result_seq.iter().cloned().for_each(|v| match v {
            Input::Undefined => (),
            Input::Value(..) => {
                flat_result.push(v);
            }
            Input::Sequence(ref seq, keep_array) => {
                if keep_array {
                    flat_result.push(v);
                } else {
                    seq.iter().cloned().for_each(|v| flat_result.push(v));
                }
            }
        });
        Ok(flat_result)
    };
}

fn evaluate_variable(name: &str, frame: &Frame) -> JsonAtaResult<Input> {
    // TODO: Something special happens when value == ""
    if let Some(binding) = frame.lookup(name) {
        // TODO: I don't like this clone
        Ok(Input::Value(binding.as_var().clone()))
    } else {
        Ok(Input::Undefined)
    }
}

fn evaluate_ternary(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    if let NodeKind::Ternary = &node.kind {
        let condition = evaluate(&node.children[0], input, frame)?;
        if boolean(&condition) {
            evaluate(&node.children[1], input, frame)
        } else {
            evaluate(&node.children[2], input, frame)
        }
    } else {
        panic!("`node` should be a NodeKind::Ternary")
    }
}

fn evaluate_name(node: &Node, input: &Input) -> JsonAtaResult<Input> {
    if let NodeKind::Name(key) = &node.kind {
        Ok(lookup(input.as_value(), key))
    } else {
        unreachable!()
    }
}

fn evaluate_block(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    if let NodeKind::Block = &node.kind {
        // TODO: block frame
        let mut result: Input = Input::Undefined;

        for child in &node.children {
            result = evaluate(child, input, frame)?;
        }

        Ok(result)
    } else {
        panic!("`node` should be a NodeKind::Block");
    }
}

fn evaluate_unary_op(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    if let NodeKind::Unary(ref op) = &node.kind {
        match op {
            UnaryOp::Minus => {
                let result = evaluate(&node.children[0], input, frame)?;
                match result {
                    Input::Value(value) => {
                        if let Some(value) = value.as_f64() {
                            Ok(Input::Value((-value).into()))
                        } else {
                            Err(Box::new(D1002 {
                                position: node.position,
                                value: value.to_string(),
                            }))
                        }
                    }
                    _ => panic!("`result` should've been an Input::Value"),
                }
            }
            UnaryOp::Array => {
                let mut result = Input::Value(array![]);
                for child in &node.children {
                    let value = evaluate(child, input, frame)?;
                    if let NodeKind::Unary(UnaryOp::Array) = child.kind {
                        result
                            .as_value_mut()
                            .push(value.as_value().clone())
                            .unwrap();
                    } else {
                        result = append(result, value);
                    }
                }
                Ok(result)
            }
            UnaryOp::Object => unimplemented!("TODO: object constructors not yet supported"),
        }
    } else {
        panic!("`node` should be a NodeKind::Unary");
    }
}

fn evaluate_binary_op(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    use BinaryOp::*;
    if let NodeKind::Binary(ref op) = &node.kind {
        match op {
            Add | Subtract | Multiply | Divide | Modulus => {
                evaluate_numeric_expression(node, input, frame, op)
            }
            LessThan | LessThanEqual | GreaterThan | GreaterThanEqual => {
                evaluate_comparison_expression(node, input, frame, op)
            }
            Equal | NotEqual => evaluate_equality_expression(node, input, frame, op),
            Concat => evaluate_string_concat(node, input, frame),
            Bind => evaluate_bind_expression(node, input, frame),
            Or | And => evaluate_boolean_expression(node, input, frame, op),
            In => evaluate_includes_expression(node, input, frame),
            _ => unimplemented!("TODO: Binary op {:?} not yet supported", op),
        }
    } else {
        panic!("`node` should be a NodeKind::Binary")
    }
}

fn evaluate_bind_expression(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    let name = &node.children[0];
    let value = evaluate(&node.children[1], input, frame)?;
    if let NodeKind::Var(name) = &name.kind {
        frame.bind(name, Binding::Var(value.as_value().clone()));
    }
    Ok(Input::Undefined)
}

fn evaluate_numeric_expression(
    node: &Node,
    input: &Input,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let lhs: f64 = match lhs.as_value() {
        JsonValue::Number(value) => value.clone().into(),
        _ => {
            return Err(Box::new(T2001 {
                position: node.position,
                op: op.to_string(),
            }))
        }
    };

    let rhs: f64 = match rhs.as_value() {
        JsonValue::Number(value) => value.clone().into(),
        _ => {
            return Err(Box::new(T2002 {
                position: node.position,
                op: op.to_string(),
            }))
        }
    };

    let result = match op {
        BinaryOp::Add => lhs + rhs,
        BinaryOp::Subtract => lhs - rhs,
        BinaryOp::Multiply => lhs * rhs,
        BinaryOp::Divide => lhs / rhs,
        BinaryOp::Modulus => lhs % rhs,
        _ => unreachable!(),
    };

    Ok(Input::Value(result.into()))
}

fn evaluate_comparison_expression(
    node: &Node,
    input: &Input,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let lhs = match lhs {
        Input::Undefined => return Ok(Input::Undefined),
        _ => lhs.as_value(),
    };

    let rhs = match rhs {
        Input::Undefined => return Ok(Input::Undefined),
        _ => rhs.as_value(),
    };

    if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
        return Err(Box::new(T2010 {
            position: node.position,
            op: op.to_string(),
        }));
    }

    if lhs.is_number() && rhs.is_number() {
        let lhs = lhs.as_f64().unwrap();
        let rhs = rhs.as_f64().unwrap();

        return Ok(Input::Value(json::from(match op {
            BinaryOp::LessThan => lhs < rhs,
            BinaryOp::LessThanEqual => lhs <= rhs,
            BinaryOp::GreaterThan => lhs > rhs,
            BinaryOp::GreaterThanEqual => lhs >= rhs,
            _ => unreachable!(),
        })));
    }

    if lhs.is_string() && rhs.is_string() {
        let lhs = lhs.as_str().unwrap();
        let rhs = rhs.as_str().unwrap();

        return Ok(Input::Value(json::from(match op {
            BinaryOp::LessThan => lhs < rhs,
            BinaryOp::LessThanEqual => lhs <= rhs,
            BinaryOp::GreaterThan => lhs > rhs,
            BinaryOp::GreaterThanEqual => lhs >= rhs,
            _ => unreachable!(),
        })));
    }

    Err(Box::new(T2009 {
        position: node.position,
        lhs: lhs.to_string(),
        rhs: rhs.to_string(),
        op: op.to_string(),
    }))
}

fn evaluate_boolean_expression(
    node: &Node,
    input: &Input,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let left_bool = boolean(&lhs);
    let right_bool = boolean(&rhs);

    let result = match op {
        BinaryOp::And => left_bool && right_bool,
        BinaryOp::Or => left_bool || right_bool,
        _ => unreachable!(),
    };

    Ok(Input::Value(result.into()))
}

fn evaluate_includes_expression(
    node: &Node,
    input: &Input,
    frame: &mut Frame,
) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    if lhs.is_value() && rhs.is_value() {
        if !rhs.as_value().is_array() {
            return Ok(Input::Value((lhs.as_value() == rhs.as_value()).into()));
        }

        for item in rhs.as_value().members() {
            if lhs.as_value() == item {
                return Ok(Input::Value(true.into()));
            }
        }
    }

    return Ok(Input::Value(false.into()));
}

fn evaluate_equality_expression(
    node: &Node,
    input: &Input,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    if lhs.is_undefined() && rhs.is_undefined() {
        return Ok(Input::Value(true.into()));
    }

    let result = match op {
        BinaryOp::Equal => lhs.as_value() == rhs.as_value(),
        BinaryOp::NotEqual => lhs.as_value() != rhs.as_value(),
        _ => unreachable!(),
    };

    Ok(Input::Value(result.into()))
}

fn evaluate_string_concat(node: &Node, input: &Input, frame: &mut Frame) -> JsonAtaResult<Input> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let lstr = if lhs.is_value() {
        lhs.as_value().as_str().unwrap_or("")
    } else {
        ""
    };

    let rstr = if rhs.is_value() {
        rhs.as_value().as_str().unwrap_or("")
    } else {
        ""
    };

    let mut result = lstr.to_owned();
    result.push_str(&rstr);

    Ok(Input::Value(result.into()))
}
