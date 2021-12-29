mod frame;
mod value;

use std::collections::HashMap;

use crate::functions::*;
use crate::{error::*, parser::ast::*, parser::Position, Result};
pub(crate) use frame::{Frame, FrameLink};
pub use value::{Value, UNDEFINED};

pub(crate) fn evaluate(node: &Node, input: &Value, frame: FrameLink) -> Result<Value> {
    let result = match node.kind {
        NodeKind::Null => Value::Null,
        NodeKind::Bool(b) => Value::Bool(b),
        NodeKind::String(ref s) => Value::String(s.clone()),
        NodeKind::Number(n) => Value::Number(n.into()),
        NodeKind::Block(ref exprs) => evaluate_block(exprs, input, frame)?,
        NodeKind::Unary(ref op) => evaluate_unary_op(node, op, input, frame)?,
        NodeKind::Binary(ref op, ref lhs, ref rhs) => {
            evaluate_binary_op(node, op, lhs, rhs, input, frame)?
        }
        NodeKind::Var(ref name) => evaluate_var(name, input, frame)?,
        NodeKind::Ternary {
            ref cond,
            ref truthy,
            ref falsy,
        } => evaluate_ternary(cond, truthy, falsy.as_deref(), input, frame)?,
        _ => unimplemented!("TODO: node kind not yet supported: {:#?}", node.kind),
    };
    Ok(result)
}

fn evaluate_block(exprs: &[Node], input: &Value, frame: FrameLink) -> Result<Value> {
    let frame = Frame::new_with_parent(frame);
    if exprs.is_empty() {
        return Ok(Value::Undefined);
    }

    let mut result = input.clone();
    for expr in exprs {
        result = evaluate(expr, &result, frame.clone())?;
    }

    Ok(result)
}

fn evaluate_var(name: &str, _input: &Value, frame: FrameLink) -> Result<Value> {
    if name.is_empty() {
        // Empty variable name returns the context value
        unimplemented!("TODO: $ context variable not implemented yet");
    } else if let Some(value) = frame.borrow().lookup(name) {
        Ok(value)
    } else {
        Ok(Value::Undefined)
    }
}

fn evaluate_ternary(
    cond: &Node,
    truthy: &Node,
    falsy: Option<&Node>,
    input: &Value,
    frame: FrameLink,
) -> Result<Value> {
    let cond = evaluate(cond, input, frame.clone())?;
    if boolean(&cond) {
        evaluate(truthy, input, frame)
    } else if let Some(falsy) = falsy {
        evaluate(falsy, input, frame)
    } else {
        Ok(Value::Undefined)
    }
}

fn evaluate_unary_op(node: &Node, op: &UnaryOp, input: &Value, frame: FrameLink) -> Result<Value> {
    match *op {
        UnaryOp::Minus(ref value) => {
            let result = evaluate(value, input, frame)?;
            match result {
                Value::Undefined => Ok(Value::Undefined),
                Value::Number(num) => Ok(Value::Number(-num)),
                _ => Err(Box::new(D1002 {
                    position: node.position,
                    value: format!("{:#?}", result),
                })),
            }
        }
        UnaryOp::ArrayConstructor(..) => {
            unimplemented!("UnaryOp::ArrayConstructor not implemented yet")
        }
        UnaryOp::ObjectConstructor(ref object) => {
            evaluate_group_expression(node.position, object, input, frame)
        }
    }
}

#[derive(Debug)]
struct Group {
    pub data: Value,
    pub index: usize,
}

type Groups = HashMap<String, Group>;

fn evaluate_group_expression(
    position: Position,
    object: &[(Node, Node)],
    input: &Value,
    frame: FrameLink,
) -> Result<Value> {
    let mut groups = Groups::new();

    if !input.is_array() {
        evaluate_group_item(position, object, input, frame.clone(), &mut groups)?;
    } else if input.is_empty() {
        evaluate_group_item(position, object, &UNDEFINED, frame.clone(), &mut groups)?;
    } else {
        for item in input.iter() {
            evaluate_group_item(position, object, item, frame.clone(), &mut groups)?;
        }
    }

    let mut result = Value::new_object();

    for key in groups.keys() {
        let group = groups.get(key).unwrap();
        let value = evaluate(&object[group.index].1, &group.data, frame.clone())?;
        if !value.is_undefined() {
            result.insert(key, value);
        }
    }

    Ok(result)
}

fn evaluate_group_item(
    position: Position,
    object: &[(Node, Node)],
    input: &Value,
    frame: FrameLink,
    groups: &mut Groups,
) -> Result<Value> {
    for (index, pair) in object.iter().enumerate() {
        let key = evaluate(&pair.0, input, frame.clone())?;
        if !key.is_string() {
            return Err(Box::new(T1003 {
                position,
                value: format!("{:#?}", key),
            }));
        }

        let key = key.as_str();

        if groups.contains_key(key) {
            if groups[key].index == index {
                return Err(Box::new(D1009 {
                    position,
                    value: key.to_owned(),
                }));
            }
            let group = groups.get_mut(key).unwrap();
            group.data = append(&group.data, input);
        } else {
            groups.insert(
                key.to_string(),
                Group {
                    data: input.clone(),
                    index,
                },
            );
        }
    }

    Ok(Value::Undefined)
}

fn evaluate_binary_op(
    node: &Node,
    op: &BinaryOp,
    lhs: &Node,
    rhs: &Node,
    input: &Value,
    frame: FrameLink,
) -> Result<Value> {
    let rhs = evaluate(&*rhs, input, frame.clone())?;

    if *op == BinaryOp::Bind {
        if let NodeKind::Var(ref name) = lhs.kind {
            frame.borrow_mut().bind(name, rhs);
        }
        return Ok(input.clone());
    }

    let lhs = evaluate(&*lhs, input, frame)?;

    match op {
        BinaryOp::Add
        | BinaryOp::Subtract
        | BinaryOp::Multiply
        | BinaryOp::Divide
        | BinaryOp::Modulus => {
            let lhs = match lhs {
                Value::Number(n) => f64::from(n),
                _ => {
                    return Err(Box::new(T2001 {
                        position: node.position,
                        op: op.to_string(),
                    }))
                }
            };

            let rhs = match rhs {
                Value::Number(n) => f64::from(n),
                _ => {
                    return Err(Box::new(T2002 {
                        position: node.position,
                        op: op.to_string(),
                    }))
                }
            };

            Ok(Value::Number(
                (match op {
                    BinaryOp::Add => lhs + rhs,
                    BinaryOp::Subtract => lhs - rhs,
                    BinaryOp::Multiply => lhs * rhs,
                    BinaryOp::Divide => lhs / rhs,
                    BinaryOp::Modulus => lhs % rhs,
                    _ => unreachable!(),
                })
                .into(),
            ))
        }

        BinaryOp::LessThan
        | BinaryOp::LessThanEqual
        | BinaryOp::GreaterThan
        | BinaryOp::GreaterThanEqual => {
            if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
                return Err(Box::new(T2010 {
                    position: node.position,
                    op: op.to_string(),
                }));
            }

            if let (Value::Number(ref lhs), Value::Number(ref rhs)) = (&lhs, &rhs) {
                let lhs = f64::from(*lhs);
                let rhs = f64::from(*rhs);
                return Ok(Value::Bool(match op {
                    BinaryOp::LessThan => lhs < rhs,
                    BinaryOp::LessThanEqual => lhs <= rhs,
                    BinaryOp::GreaterThan => lhs > rhs,
                    BinaryOp::GreaterThanEqual => lhs >= rhs,
                    _ => unreachable!(),
                }));
            }

            if let (Value::String(ref lhs), Value::String(ref rhs)) = (&lhs, &rhs) {
                return Ok(Value::Bool(match op {
                    BinaryOp::LessThan => lhs < rhs,
                    BinaryOp::LessThanEqual => lhs <= rhs,
                    BinaryOp::GreaterThan => lhs > rhs,
                    BinaryOp::GreaterThanEqual => lhs >= rhs,
                    _ => unreachable!(),
                }));
            }

            Err(Box::new(T2009 {
                position: node.position,
                lhs: format!("{:#?}", lhs),
                rhs: format!("{:#?}", rhs),
                op: op.to_string(),
            }))
        }

        BinaryOp::Equal | BinaryOp::NotEqual => {
            if lhs.is_undefined() || rhs.is_undefined() {
                return Ok(Value::Bool(false));
            }

            Ok(Value::Bool(match op {
                BinaryOp::Equal => lhs == rhs,
                BinaryOp::NotEqual => lhs != rhs,
                _ => unreachable!(),
            }))
        }

        _ => unimplemented!("TODO: binary op not supported yet: {:#?}", *op),
    }
}
