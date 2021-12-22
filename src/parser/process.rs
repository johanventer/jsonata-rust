// use crate::error::*;
use crate::Result;

use super::{ast::*, Position};
use crate::error::*;

pub fn process_ast(node: Node) -> Result<Node> {
    let mut node = node;
    let keep_array = node.keep_array;

    let mut result = match node.kind {
        NodeKind::Name(..) => process_name(node)?,
        NodeKind::Block(..) => process_block(node)?,
        NodeKind::Unary(..) => process_unary(node)?,
        NodeKind::Binary(..) => process_binary(node)?,
        NodeKind::GroupBy(ref mut lhs, ref mut rhs) => process_group_by(node.position, lhs, rhs)?,
        NodeKind::OrderBy(ref mut lhs, ref mut rhs) => process_order_by(node.position, lhs, rhs)?,
        NodeKind::Function { .. } => unimplemented!("Function not yet implemented"),
        NodeKind::Lambda { .. } => unimplemented!("Lambda not yet implemented"),
        NodeKind::Ternary { .. } => unimplemented!("Ternary not yet implemented"),
        NodeKind::Transform { .. } => unimplemented!("Transform not yet implemented"),
        NodeKind::Parent => unimplemented!("Parent not yet implemented"),
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
            if let NodeKind::Number(ref mut num) = result.kind {
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
        NodeKind::Binary(BinaryOp::Map, ref mut lhs, ref mut rhs) => {
            process_path(node.position, lhs, rhs)
        }
        NodeKind::Binary(BinaryOp::Predicate, ref mut lhs, ref mut rhs) => {
            process_predicate(node.position, lhs, rhs)
        }
        NodeKind::Binary(BinaryOp::Bind, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("Bind not yet implemented")
        }
        NodeKind::Binary(BinaryOp::ContextBind, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("ContextBind not yet implemented")
        }
        NodeKind::Binary(BinaryOp::PositionalBind, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("PositionBind not yet implemented")
        }
        NodeKind::Binary(BinaryOp::Apply, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("Apply not yet implemented")
        }
        NodeKind::Binary(_, ref mut lhs, ref mut rhs) => {
            *lhs = Box::new(process_ast(std::mem::take(lhs))?);
            *rhs = Box::new(process_ast(std::mem::take(rhs))?);
            Ok(node)
        }
        _ => unreachable!(),
    }
}

fn process_path(position: Position, lhs: &mut Box<Node>, rhs: &mut Box<Node>) -> Result<Node> {
    let left_step = process_ast(std::mem::take(lhs))?;
    let mut rest = process_ast(std::mem::take(rhs))?;

    // If the left_step is a path itself, start with that. Otherwise, start a new path
    let mut result = if matches!(left_step.kind, NodeKind::Path(_)) {
        left_step
    } else {
        Node::new(NodeKind::Path(vec![left_step]), position)
    };

    // TODO: If the lhs is a Parent (parser.js:997)
    // TODO: If the rhs is a Function (parser.js:1001)

    if let NodeKind::Path(ref mut steps) = result.kind {
        if let NodeKind::Path(ref mut rest_steps) = rest.kind {
            // If the rest is a path, merge in the steps
            steps.append(rest_steps);
        } else {
            // If there are predicates on the rest, they become stages of the step
            rest.stages = rest.predicates.take();
            steps.push(rest);
        }

        let mut keep_singleton_array = false;
        let last_index = steps.len() - 1;

        for (step_index, step) in steps.iter_mut().enumerate() {
            match step.kind {
                // Steps can't be literal values other than strings
                NodeKind::Number(..) | NodeKind::Bool(..) | NodeKind::Null => {
                    return Err(Box::new(S0213 {
                        position: step.position,
                        value: format!("{:#?}", step.kind),
                    }));
                }

                // Steps that are string literals should become Names
                NodeKind::String(ref s) => {
                    step.kind = NodeKind::Name(s.clone());
                }

                // If the first or last step is an array constructor, it shouldn't be flattened
                NodeKind::Unary(UnaryOp::ArrayConstructor(..)) => {
                    if step_index == 0 || step_index == last_index {
                        step.cons_array = true;
                    }
                }

                _ => (),
            }

            // Any step that signals keeping a singleton array should be plagged on the path
            keep_singleton_array = keep_singleton_array || step.keep_array;
        }

        result.keep_singleton_array = keep_singleton_array;
    }

    Ok(result)
}

fn process_predicate(position: Position, lhs: &mut Box<Node>, rhs: &mut Box<Node>) -> Result<Node> {
    let mut result = process_ast(std::mem::take(lhs))?;
    let mut in_path = false;

    let node = if let NodeKind::Path(ref mut steps) = result.kind {
        in_path = true;
        let last_index = steps.len() - 1;
        &mut steps[last_index]
    } else {
        &mut result
    };

    // Predicates can't follow group-by
    if node.group_by.is_some() {
        return Err(Box::new(S0209 { position }));
    }

    let filter = Node::new(
        NodeKind::Filter(Box::new(process_ast(std::mem::take(rhs))?)),
        position,
    );

    // TODO: seekingParent (parser.js:1074)

    // Add the filter to the node. If it's a step in a path, it goes in stages, otherwise in predicated
    if in_path {
        match node.stages {
            None => node.stages = Some(vec![filter]),
            Some(ref mut stages) => {
                stages.push(filter);
            }
        }
    } else {
        match node.predicates {
            None => node.predicates = Some(vec![filter]),
            Some(ref mut predicates) => {
                predicates.push(filter);
            }
        }
    }

    Ok(result)
}

fn process_group_by(position: Position, lhs: &mut Box<Node>, rhs: &mut Object) -> Result<Node> {
    let mut result = process_ast(std::mem::take(lhs))?;

    // Can only have a single grouping expression
    if result.group_by.is_some() {
        return Err(Box::new(S0210 { position }));
    }

    // Process all the key, value pairs
    for pair in rhs.iter_mut() {
        let key = std::mem::take(&mut pair.0);
        let value = std::mem::take(&mut pair.1);
        *pair = (process_ast(key)?, process_ast(value)?);
    }

    result.group_by = Some((position, std::mem::take(rhs)));

    Ok(result)
}

fn process_order_by(position: Position, lhs: &mut Box<Node>, rhs: &mut SortTerms) -> Result<Node> {
    let lhs = process_ast(std::mem::take(lhs))?;

    // If the left hand side is not a path, make it one
    let mut result = if matches!(lhs.kind, NodeKind::Path(_)) {
        lhs
    } else {
        Node::new(NodeKind::Path(vec![lhs]), position)
    };

    // Process all the sort terms
    for pair in rhs.iter_mut() {
        *pair = (process_ast(std::mem::take(&mut pair.0))?, pair.1);
    }

    if let NodeKind::Path(ref mut steps) = result.kind {
        steps.push(Node::new(NodeKind::Sort(std::mem::take(rhs)), position));
    }

    Ok(result)
}

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

/*
    keep_array is used on individual nodes
    keep_singleton_array is used on Paths
    cons_array is for special handling of paths that start or end with an array constructor
    predicates is used on individual nodes
    stages are used in steps in a Path
*/
