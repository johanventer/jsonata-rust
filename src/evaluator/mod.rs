mod frame;
mod value;

use crate::functions::*;
use crate::{error::*, parser::ast::*, Result};
pub(crate) use frame::{Frame, FrameLink};
pub use value::Value;

pub(crate) fn evaluate(node: &Node, input: &Value, frame: FrameLink) -> Result<Value> {
    let result = match node.kind {
        NodeKind::Null => Value::Null,
        NodeKind::Bool(b) => Value::Bool(b),
        NodeKind::String(ref s) => Value::String(s.clone()),
        NodeKind::Number(n) => Value::Number(n),
        NodeKind::Block(ref exprs) => evaluate_block(exprs, input, frame)?,
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
                Value::Number(n) => n,
                _ => {
                    return Err(Box::new(T2001 {
                        position: node.position,
                        op: op.to_string(),
                    }))
                }
            };

            let rhs = match rhs {
                Value::Number(n) => n,
                _ => {
                    return Err(Box::new(T2002 {
                        position: node.position,
                        op: op.to_string(),
                    }))
                }
            };

            Ok(Value::Number(match op {
                BinaryOp::Add => lhs + rhs,
                BinaryOp::Subtract => lhs - rhs,
                BinaryOp::Multiply => lhs * rhs,
                BinaryOp::Divide => lhs / rhs,
                BinaryOp::Modulus => lhs % rhs,
                _ => unreachable!(),
            }))
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

            if let (Value::Number(lhs), Value::Number(rhs)) = (&lhs, &rhs) {
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
