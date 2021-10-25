use crate::error::*;
use crate::Result;

use super::ast::*;
use super::Position;

pub(super) fn process_ast(node: Box<Node>) -> Result<Box<Node>> {
    let keep_array = node.keep_array;

    let mut result = match node.kind {
        NodeKind::Name(..) => process_name(node)?,
        NodeKind::Block(..) => process_block(node)?,
        NodeKind::Unary(..) => process_unary(node)?,
        NodeKind::Binary(..) => process_binary(node)?,
        //     NodeKind::Binary(BinaryOp::GroupBy(..)) => process_group_by(node),
        //     NodeKind::Binary(BinaryOp::SortOp(..)) => process_sort(node),
        NodeKind::Lambda { .. } => process_lambda(node)?,

        //     Function
        //     Ternary
        //     Transform
        _ => node,
    };

    if keep_array {
        result.keep_array = true;
    }

    Ok(result)
}

#[inline]
fn process_name(node: Box<Node>) -> Result<Box<Node>> {
    let position = node.position;
    let keep_singleton_array = node.keep_array;
    let mut result = Box::new(Node::new_path(position, vec![node]));
    result.keep_singleton_array = keep_singleton_array;
    Ok(result)
}

#[inline]
fn process_block(mut node: Box<Node>) -> Result<Box<Node>> {
    let cons_array = false;
    if let NodeKind::Block(ref mut exprs) = node.kind {
        *exprs = exprs
            .drain(..)
            .map(|expr| process_ast(expr))
            .collect::<Result<Vec<Box<Node>>>>()?;
    }
    node.cons_array = cons_array;
    Ok(node)
}

#[inline]
fn process_unary(mut node: Box<Node>) -> Result<Box<Node>> {
    match node.kind {
        NodeKind::Unary(UnaryOp::Minus(value)) => {
            let mut result = process_ast(value)?;
            return if let NodeKind::Num(ref mut num) = result.kind {
                // Pre-process unary minus on numbers by negating the number
                *num = -*num;
                Ok(result)
            } else {
                Ok(Box::new(Node::new(
                    NodeKind::Unary(UnaryOp::Minus(result)),
                    node.position,
                )))
            };
        }

        NodeKind::Unary(UnaryOp::ArrayConstructor(ref mut exprs)) => {
            *exprs = exprs
                .drain(..)
                .map(|expr| process_ast(expr))
                .collect::<Result<Vec<Box<Node>>>>()?;
            Ok(node)
        }

        NodeKind::Unary(UnaryOp::ObjectConstructor(ref mut object)) => {
            *object = object
                .drain(0..)
                .map(|(k, v)| Ok((process_ast(k)?, process_ast(v)?)))
                .collect::<Result<Vec<(Box<Node>, Box<Node>)>>>()?;
            Ok(node)
        }

        _ => unreachable!(),
    }
}

#[inline]
fn process_binary(mut node: Box<Node>) -> Result<Box<Node>> {
    match node.kind {
        NodeKind::Binary(BinaryOp::Path, lhs, rhs) => process_path(lhs, rhs),
        NodeKind::Binary(BinaryOp::Predicate, lhs, rhs) => {
            process_predicate(node.position, lhs, rhs)
        }
        // TODO ContextBind & PositionalBind need more processing
        NodeKind::Binary(op, lhs, rhs) => {
            node.kind = NodeKind::Binary(op, process_ast(lhs)?, process_ast(rhs)?);
            Ok(node)
        }
        _ => unreachable!(),
    }
}

#[inline]
fn process_path(lhs: Box<Node>, rhs: Box<Node>) -> Result<Box<Node>> {
    let lhs = process_ast(lhs)?;
    let mut rhs = process_ast(rhs)?;

    let mut result = {
        // If lhs is a Path, start with that, otherwise create a new one
        if lhs.is_path() {
            lhs
        } else {
            Box::new(Node::new_path(lhs.position, vec![lhs]))
        }
    };

    // TODO: If the lhs is a Parent (parser.js:997)

    // TODO: If the rhs is a Function (parser.js:1001)

    // If rhs is a Path, merge the steps in
    if rhs.is_path() {
        result.append_steps(&mut rhs.take_path_steps());
    } else {
        if rhs.predicates.is_some() {
            rhs.stages = rhs.predicates;
            rhs.predicates = None;
        }
        result.push_step(rhs);
    }

    let last_index = result.path_len() - 1;
    let mut keep_singleton_array = false;

    for (step_index, step) in result.path_steps().iter_mut().enumerate() {
        match step.kind {
            // Steps cannot be literal values
            NodeKind::Num(..) | NodeKind::Bool(..) | NodeKind::Null => {
                return Err(Box::new(S0213 {
                    position: step.position,
                    value: step.kind.to_string(),
                }))
            }
            // Steps that are string literals should be switched to Name
            NodeKind::Str(ref v) => {
                step.kind = NodeKind::Name(v.clone());
            }
            // If first or last step is an array constructor, it shouldn't be flattened
            NodeKind::Unary(ref op) => {
                if matches!(op, UnaryOp::ArrayConstructor(..))
                    && (step_index == 0 || step_index == last_index)
                {
                    step.cons_array = true;
                }
            }
            _ => (),
        }

        keep_singleton_array = keep_singleton_array || step.keep_array;
    }

    result.keep_singleton_array = keep_singleton_array;

    Ok(result)
}

#[inline]
fn process_predicate(position: Position, lhs: Box<Node>, rhs: Box<Node>) -> Result<Box<Node>> {
    let mut result = process_ast(lhs)?;
    let mut is_stages = false;

    let step = if result.is_path() {
        is_stages = true;
        let last_index = result.path_len() - 1;
        &mut result.path_steps()[last_index]
    } else {
        &mut result
    };

    if step.group_by.is_some() {
        return Err(Box::new(S0209 { position }));
    }

    let predicate = process_ast(rhs)?;

    // TODO: seekingParent (parser.js:1074)

    if is_stages {
        if step.stages.is_none() {
            step.stages = Some(vec![predicate]);
        } else {
            if let Some(ref mut stages) = step.stages {
                stages.push(predicate);
            }
        }
    } else {
        if step.predicates.is_none() {
            step.predicates = Some(vec![predicate]);
        } else {
            if let Some(ref mut predicates) = step.predicates {
                predicates.push(predicate);
            }
        }
    }

    Ok(result)
}

#[inline]
fn process_lambda(mut node: Box<Node>) -> Result<Box<Node>> {
    if let NodeKind::Lambda { args, body, .. } = node.kind {
        //let body = tail_call_optimize(process_ast(body)?)?;

        node.kind = NodeKind::Lambda {
            args,
            body: process_ast(body)?.into(),
        };

        Ok(node)
    } else {
        unreachable!()
    }
}

// fn tail_call_optimize(mut node: Box<Node>) -> Result<Box<Node>> {
//     match node.kind {
//         NodeKind::Function { .. } if node.predicates.is_none() => {
//             let position = node.position;
//             Ok(Box::new(Node::new(
//                 NodeKind::Lambda {
//                     args: Rc::new(Vec::new()),
//                     body: node.into(),
//                     thunk: true,
//                 },
//                 position,
//             )))
//         }
//         NodeKind::Ternary {
//             cond,
//             truthy,
//             falsy,
//         } => {
//             node.kind = NodeKind::Ternary {
//                 cond,
//                 truthy: tail_call_optimize(truthy)?,
//                 falsy: match falsy {
//                     None => None,
//                     Some(falsy) => Some(tail_call_optimize(falsy)?),
//                 },
//             };
//             Ok(node)
//         }
//         NodeKind::Block(ref mut exprs) => {
//             let len = exprs.len();
//             if len > 0 {
//                 let last = tail_call_optimize(exprs.pop().unwrap())?;
//                 exprs.push(last);
//             }
//             Ok(node)
//         }
//         _ => Ok(node),
//     }
// }
