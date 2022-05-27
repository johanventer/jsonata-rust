use jsonata_errors::{Error, Result};

use super::*;

impl Ast {
    pub fn process(self) -> Result<Ast> {
        process_ast(self)
    }
}

pub fn process_ast(node: Ast) -> Result<Ast> {
    let mut node = node;
    let keep_array = node.keep_array;

    let mut result = match node.kind {
        AstKind::Name(..) => process_name(node)?,
        AstKind::Block(..) => process_block(node)?,
        AstKind::Unary(..) => process_unary(node)?,
        AstKind::Binary(..) => process_binary(node)?,
        AstKind::GroupBy(ref mut lhs, ref mut rhs) => process_group_by(node.char_index, lhs, rhs)?,
        AstKind::OrderBy(ref mut lhs, ref mut rhs) => process_order_by(node.char_index, lhs, rhs)?,
        AstKind::Function {
            ref mut proc,
            ref mut args,
            ..
        } => {
            process_function(proc, args)?;
            node
        }
        AstKind::Lambda { ref mut body, .. } => {
            process_lambda(body)?;
            node
        }
        AstKind::Ternary { .. } => process_ternary(node)?,
        AstKind::Transform { .. } => process_transform(node)?,
        AstKind::Parent => unimplemented!("Parent not yet implemented"),
        _ => node,
    };

    if keep_array {
        result.keep_array = true;
    }

    Ok(result)
}

// Turn a Name into a Path with a single step
fn process_name(node: Ast) -> Result<Ast> {
    let char_index = node.char_index;
    let keep_singleton_array = node.keep_array;
    let mut result = Ast::new(AstKind::Path(vec![node]), char_index);
    result.keep_singleton_array = keep_singleton_array;
    Ok(result)
}

// Process each expression in a block
fn process_block(node: Ast) -> Result<Ast> {
    let mut node = node;
    if let AstKind::Block(ref mut exprs) = node.kind {
        for expr in exprs {
            *expr = process_ast(std::mem::take(expr))?;
        }
    }
    Ok(node)
}

fn process_ternary(node: Ast) -> Result<Ast> {
    let mut node = node;
    if let AstKind::Ternary {
        ref mut cond,
        ref mut truthy,
        ref mut falsy,
    } = node.kind
    {
        *cond = Box::new(process_ast(std::mem::take(cond))?);
        *truthy = Box::new(process_ast(std::mem::take(truthy))?);
        if let Some(ref mut falsy) = falsy {
            *falsy = Box::new(process_ast(std::mem::take(falsy))?);
        }
    } else {
        unreachable!()
    }

    Ok(node)
}

fn process_transform(node: Ast) -> Result<Ast> {
    let mut node = node;
    if let AstKind::Transform {
        ref mut pattern,
        ref mut update,
        ref mut delete,
    } = node.kind
    {
        *pattern = Box::new(process_ast(std::mem::take(pattern))?);
        *update = Box::new(process_ast(std::mem::take(update))?);
        if let Some(ref mut delete) = delete {
            *delete = Box::new(process_ast(std::mem::take(delete))?);
        }
    }

    Ok(node)
}

fn process_unary(node: Ast) -> Result<Ast> {
    let mut node = node;

    match node.kind {
        // Pre-process negative numbers
        AstKind::Unary(UnaryOp::Minus(value)) => {
            let mut result = process_ast(*value)?;
            match result.kind {
                AstKind::Unsigned(ref v) => {
                    let v = -(*v as i64);
                    Ok(Ast::new(AstKind::Signed(v), node.char_index))
                }
                AstKind::Float(ref mut v) => {
                    *v = -*v;
                    Ok(result)
                }
                _ => Ok(Ast::new(
                    AstKind::Unary(UnaryOp::Minus(Box::new(result))),
                    node.char_index,
                )),
            }
        }

        // Process all of the expressions in an array constructor
        AstKind::Unary(UnaryOp::ArrayConstructor(ref mut exprs)) => {
            for expr in exprs {
                *expr = process_ast(std::mem::take(expr))?;
            }
            Ok(node)
        }

        // Process all the keys and values in an object constructor
        AstKind::Unary(UnaryOp::ObjectConstructor(ref mut object)) => {
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

fn process_binary(node: Ast) -> Result<Ast> {
    let mut node = node;

    match node.kind {
        AstKind::Binary(BinaryOp::Map, ref mut lhs, ref mut rhs) => {
            process_path(node.char_index, lhs, rhs)
        }
        AstKind::Binary(BinaryOp::Predicate, ref mut lhs, ref mut rhs) => {
            process_predicate(node.char_index, lhs, rhs)
        }
        AstKind::Binary(BinaryOp::ContextBind, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("ContextBind not yet implemented")
        }
        AstKind::Binary(BinaryOp::PositionalBind, ref mut _lhs, ref mut _rhs) => {
            unimplemented!("PositionBind not yet implemented")
        }
        AstKind::Binary(_, ref mut lhs, ref mut rhs) => {
            *lhs = Box::new(process_ast(std::mem::take(lhs))?);
            *rhs = Box::new(process_ast(std::mem::take(rhs))?);
            Ok(node)
        }
        _ => unreachable!(),
    }
}

fn process_path(char_index: usize, lhs: &mut Box<Ast>, rhs: &mut Box<Ast>) -> Result<Ast> {
    let left_step = process_ast(std::mem::take(lhs))?;
    let mut rest = process_ast(std::mem::take(rhs))?;

    // If the left_step is a path itself, start with that. Otherwise, start a new path
    let mut result = if matches!(left_step.kind, AstKind::Path(_)) {
        left_step
    } else {
        Ast::new(AstKind::Path(vec![left_step]), char_index)
    };

    // TODO: If the lhs is a Parent (parser.js:997)
    // TODO: If the rhs is a Function (parser.js:1001)

    if let AstKind::Path(ref mut steps) = result.kind {
        if let AstKind::Path(ref mut rest_steps) = rest.kind {
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
                AstKind::Unsigned(..)
                | AstKind::Signed(..)
                | AstKind::Float(..)
                | AstKind::Bool(..)
                | AstKind::Null => {
                    return Err(Error::S0213InvalidStep(step.char_index, "TODO".to_string()));
                }

                // Steps that are string literals should become Names
                AstKind::String(ref s) => {
                    step.kind = AstKind::Name(s.clone());
                }

                // If the first or last step is an array constructor, it shouldn't be flattened
                AstKind::Unary(UnaryOp::ArrayConstructor(..)) => {
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

fn process_predicate(char_index: usize, lhs: &mut Box<Ast>, rhs: &mut Box<Ast>) -> Result<Ast> {
    let mut result = process_ast(std::mem::take(lhs))?;
    let mut in_path = false;

    let node = if let AstKind::Path(ref mut steps) = result.kind {
        in_path = true;
        let last_index = steps.len() - 1;
        &mut steps[last_index]
    } else {
        &mut result
    };

    // Predicates can't follow group-by
    if node.group_by.is_some() {
        return Err(Error::S0209InvalidPredicate(char_index));
    }

    let filter = Ast::new(
        AstKind::Filter(Box::new(process_ast(std::mem::take(rhs))?)),
        char_index,
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

fn process_group_by(char_index: usize, lhs: &mut Box<Ast>, rhs: &mut Object) -> Result<Ast> {
    let mut result = process_ast(std::mem::take(lhs))?;

    // Can only have a single grouping expression
    if result.group_by.is_some() {
        return Err(Error::S0210MultipleGroupBy(char_index));
    }

    // Process all the key, value pairs
    for pair in rhs.iter_mut() {
        let key = std::mem::take(&mut pair.0);
        let value = std::mem::take(&mut pair.1);
        *pair = (process_ast(key)?, process_ast(value)?);
    }

    result.group_by = Some((char_index, std::mem::take(rhs)));

    Ok(result)
}

fn process_order_by(char_index: usize, lhs: &mut Box<Ast>, rhs: &mut SortTerms) -> Result<Ast> {
    let lhs = process_ast(std::mem::take(lhs))?;

    // If the left hand side is not a path, make it one
    let mut result = if matches!(lhs.kind, AstKind::Path(_)) {
        lhs
    } else {
        Ast::new(AstKind::Path(vec![lhs]), char_index)
    };

    // Process all the sort terms
    for pair in rhs.iter_mut() {
        *pair = (process_ast(std::mem::take(&mut pair.0))?, pair.1);
    }

    if let AstKind::Path(ref mut steps) = result.kind {
        steps.push(Ast::new(AstKind::Sort(std::mem::take(rhs)), char_index));
    }

    Ok(result)
}

fn process_function(proc: &mut Box<Ast>, args: &mut [Ast]) -> Result<()> {
    *proc = Box::new(process_ast(std::mem::take(&mut *proc))?);
    for arg in args.iter_mut() {
        *arg = process_ast(std::mem::take(arg))?;
    }
    Ok(())
}

fn process_lambda(body: &mut Box<Ast>) -> Result<()> {
    let new_body = process_ast(std::mem::take(body))?;
    let new_body = tail_call_optimize(new_body)?;
    *body = Box::new(new_body);
    Ok(())
}

fn tail_call_optimize(mut expr: Ast) -> Result<Ast> {
    match &mut expr.kind {
        AstKind::Function { .. } if expr.predicates.is_none() => {
            let char_index = expr.char_index;
            let thunk = Ast::new(
                AstKind::Lambda {
                    name: String::from("thunk"),
                    args: vec![],
                    thunk: true,
                    body: Box::new(expr),
                },
                char_index,
            );

            Ok(thunk)
        }
        AstKind::Ternary { truthy, falsy, .. } => {
            *truthy = Box::new(tail_call_optimize(std::mem::take(truthy))?);
            match falsy {
                Some(inner) => *falsy = Some(Box::new(tail_call_optimize(std::mem::take(inner))?)),
                None => {}
            }
            Ok(expr)
        }
        AstKind::Block(statements) => {
            let length = statements.len();
            if length > 0 {
                statements[length - 1] =
                    tail_call_optimize(std::mem::take(&mut statements[length - 1]))?;
            }
            Ok(expr)
        }
        _ => Ok(expr),
    }
}

/*
    keep_array is used on individual nodes
    keep_singleton_array is used on Paths
    cons_array is for special handling of paths that start or end with an array constructor
    predicates is used on individual nodes
    stages are used in steps in a Path
*/
