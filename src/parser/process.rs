// use crate::error::*;
use crate::Result;

use super::ast::*;
// use super::Position;

pub fn process_ast(node: Node) -> Result<Node> {
    let keep_array = node.keep_array;

    let mut result = match node.kind {
        NodeKind::Name(..) => process_name(node)?,
        NodeKind::Block(..) => process_block(node)?,
        NodeKind::Unary(..) => process_unary(node)?,
        NodeKind::Binary(..) => process_binary(node)?,
        NodeKind::GroupBy(..) => node,      // process_group_by(node),
        NodeKind::OrderBy(..) => node,      // process_sort(node),
        NodeKind::Function { .. } => node,  // TODO
        NodeKind::Lambda { .. } => node,    // process_lambda(node)?,
        NodeKind::Ternary { .. } => node,   // TODO
        NodeKind::Transform { .. } => node, // TODO
        NodeKind::Parent => node,           // TODO
        _ => node,
    };

    if keep_array {
        result.keep_array = true;
    }

    Ok(result)
}

// Turn a Name into a Path with a single step
fn process_name(node: Node) -> Result<Node> {
    let position = node.position;
    let keep_singleton_array = node.keep_array;
    let mut result = Node::new(NodeKind::Path(vec![node]), position);
    result.keep_singleton_array = keep_singleton_array;
    Ok(result)
}

// Process each expression in a block
fn process_block(node: Node) -> Result<Node> {
    let mut node = node;
    if let NodeKind::Block(ref mut exprs) = node.kind {
        for expr in exprs {
            *expr = process_ast(std::mem::take(expr))?;
        }
    }
    Ok(node)
}

fn process_unary(node: Node) -> Result<Node> {
    let mut node = node;

    match node.kind {
        // Pre-process negative numbers
        NodeKind::Unary(UnaryOp::Minus(value)) => {
            let mut result = process_ast(*value)?;
            if let NodeKind::Num(ref mut num) = result.kind {
                *num = -*num;
                Ok(result)
            } else {
                Ok(Node::new(
                    NodeKind::Unary(UnaryOp::Minus(Box::new(result))),
                    node.position,
                ))
            }
        }

        // Process all of the expressions in an array constructor
        NodeKind::Unary(UnaryOp::ArrayConstructor(ref mut exprs)) => {
            for expr in exprs {
                *expr = process_ast(std::mem::take(expr))?;
            }
            Ok(node)
        }

        // Process all the keys and values in an object constructor
        NodeKind::Unary(UnaryOp::ObjectConstructor(ref mut object)) => {
            for pair in object {
                let key = std::mem::take(&mut pair.0);
                let value = std::mem::take(&mut pair.1);
                *pair = (process_ast(key)?, process_ast(value)?);
            }
            Ok(node)
        }
        _ => unreachable!(),
    }
}

fn process_binary(node: Node) -> Result<Node> {
    let mut node = node;

    match node.kind {
        NodeKind::Binary(BinaryOp::Map, ref mut _lhs, ref mut _rhs) => Ok(node), //process_path(lhs, rhs),
        NodeKind::Binary(BinaryOp::Predicate, ref mut _lhs, ref mut _rhs) => Ok(node), // process_predicate(node.position, lhs, rhs)
        NodeKind::Binary(BinaryOp::ContextBind, ref mut _lhs, ref mut _rhs) => Ok(node), // TODO
        NodeKind::Binary(BinaryOp::PositionalBind, ref mut _lhs, ref mut _rhs) => Ok(node), // TODO
        NodeKind::Binary(_, ref mut lhs, ref mut rhs) => {
            *lhs = Box::new(process_ast(std::mem::take(lhs))?);
            *rhs = Box::new(process_ast(std::mem::take(rhs))?);
            Ok(node)
        }
        _ => unreachable!(),
    }
}

// #[inline]
// fn process_path(lhs: Box<Node>, rhs: Box<Node>) -> Result<Box<Node>> {
//     let lhs = process_ast(lhs)?;
//     let mut rhs = process_ast(rhs)?;

//     let mut result = {
//         // If lhs is a Path, start with that, otherwise create a new one
//         if lhs.is_path() {
//             lhs
//         } else {
//             Box::new(Node::new_path(lhs.position, vec![lhs]))
//         }
//     };

//     // TODO: If the lhs is a Parent (parser.js:997)

//     // TODO: If the rhs is a Function (parser.js:1001)

//     // If rhs is a Path, merge the steps in
//     if rhs.is_path() {
//         result.append_steps(&mut rhs.take_path_steps());
//     } else {
//         if rhs.predicates.is_some() {
//             rhs.stages = rhs.predicates;
//             rhs.predicates = None;
//         }
//         result.push_step(rhs);
//     }

//     let last_index = result.path_len() - 1;
//     let mut keep_singleton_array = false;

//     for (step_index, step) in result.path_steps().iter_mut().enumerate() {
//         match step.kind {
//             // Steps cannot be literal values
//             NodeKind::Num(..) | NodeKind::Bool(..) | NodeKind::Null => {
//                 return Err(Box::new(S0213 {
//                     position: step.position,
//                     value: step.kind.to_string(),
//                 }))
//             }
//             // Steps that are string literals should be switched to Name
//             NodeKind::Str(ref v) => {
//                 step.kind = NodeKind::Name(v.clone());
//             }
//             // If first or last step is an array constructor, it shouldn't be flattened
//             NodeKind::Unary(ref op) => {
//                 if matches!(op, UnaryOp::ArrayConstructor(..))
//                     && (step_index == 0 || step_index == last_index)
//                 {
//                     step.cons_array = true;
//                 }
//             }
//             _ => (),
//         }

//         keep_singleton_array = keep_singleton_array || step.keep_array;
//     }

//     result.keep_singleton_array = keep_singleton_array;

//     Ok(result)
// }

// #[inline]
// fn process_predicate(position: Position, lhs: Box<Node>, rhs: Box<Node>) -> Result<Box<Node>> {
//     let mut result = process_ast(lhs)?;
//     let mut is_stages = false;

//     let step = if result.is_path() {
//         is_stages = true;
//         let last_index = result.path_len() - 1;
//         &mut result.path_steps()[last_index]
//     } else {
//         &mut result
//     };

//     if step.group_by.is_some() {
//         return Err(Box::new(S0209 { position }));
//     }

//     let predicate = process_ast(rhs)?;

//     // TODO: seekingParent (parser.js:1074)

//     if is_stages {
//         if step.stages.is_none() {
//             step.stages = Some(vec![predicate]);
//         } else {
//             if let Some(ref mut stages) = step.stages {
//                 stages.push(predicate);
//             }
//         }
//     } else {
//         if step.predicates.is_none() {
//             step.predicates = Some(vec![predicate]);
//         } else {
//             if let Some(ref mut predicates) = step.predicates {
//                 predicates.push(predicate);
//             }
//         }
//     }

//     Ok(result)
// }

// #[inline]
// fn process_lambda(mut node: Box<Node>) -> Result<Box<Node>> {
//     if let NodeKind::Lambda { args, body, .. } = node.kind {
//         //let body = tail_call_optimize(process_ast(body)?)?;

//         node.kind = NodeKind::Lambda {
//             args,
//             body: process_ast(body)?.into(),
//         };

//         Ok(node)
//     } else {
//         unreachable!()
//     }
// }

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
