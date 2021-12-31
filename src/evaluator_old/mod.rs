pub mod frame;

use crate::{Error, Position, Result, Value, ValuePool};
use std::collections::HashMap;

pub(crate) fn evaluate(node: &Node, input: &Value, frame: FrameLink) -> Result<Value> {
    let mut result = match node.kind {
        NodeKind::Null => Value::Null,
        NodeKind::Bool(b) => Value::Bool(b),
        NodeKind::String(ref s) => Value::String(s.clone()),
        NodeKind::Number(n) => Value::Number(n.into()),
        NodeKind::Block(ref exprs) => evaluate_block(exprs, input, frame.clone())?,
        NodeKind::Unary(ref op) => evaluate_unary_op(node, op, input, frame.clone())?,
        NodeKind::Binary(ref op, ref lhs, ref rhs) => {
            evaluate_binary_op(node, op, lhs, rhs, input, frame.clone())?
        }
        NodeKind::Var(ref name) => evaluate_var(name, input, frame.clone())?,
        NodeKind::Ternary {
            ref cond,
            ref truthy,
            ref falsy,
        } => evaluate_ternary(cond, truthy, falsy.as_deref(), input, frame.clone())?,
        NodeKind::Path(ref steps) => evaluate_path(node, steps, input, frame.clone())?,
        NodeKind::Name(ref name) => todo!("Reimplemnt lookup"), //lookup(input, name),
        _ => unimplemented!("TODO: node kind not yet supported: {:#?}", node.kind),
    };

    if let Some(filters) = &node.predicates {
        for filter in filters {
            result = evaluate_filter(filter, &result, frame.clone())?;
        }
    }

    Ok(
        if let Value::Array {
            is_sequence: true,
            ref mut items,
            ref mut keep_singleton,
            ..
        } = result
        {
            if node.keep_array {
                *keep_singleton = true;
            }
            if items.is_empty() {
                Value::Undefined
            } else if items.len() == 1 {
                if *keep_singleton {
                    result
                } else {
                    std::mem::take(&mut items[0])
                }
            } else {
                result
            }
        } else {
            result
        },
    )
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
        UnaryOp::ArrayConstructor(ref array) => {
            let mut result: Vec<Value> = Vec::with_capacity(array.len());
            for item in array.iter() {
                let value = evaluate(item, input, frame.clone())?;
                result.push(value);
            }
            Ok(Value::Array {
                items: result,
                is_sequence: false,
                cons: node.cons_array,
                keep_singleton: false,
            })
        }
        UnaryOp::ObjectConstructor(ref object) => {
            evaluate_group_expression(node.position, object, input, frame)
        }
    }
}

fn evaluate_group_expression(
    position: Position,
    object: &[(Node, Node)],
    input: &Value,
    frame: FrameLink,
) -> Result<Value> {
    struct Group {
        pub data: Value,
        pub index: usize,
    }

    let mut groups: HashMap<String, Group> = HashMap::new();

    let mut evaluate_group_item = |item: &Value| -> Result<Value> {
        for (index, pair) in object.iter().enumerate() {
            let key = evaluate(&pair.0, item, frame.clone())?;
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
                group.data = append(&group.data, item);
            } else {
                groups.insert(
                    key.to_string(),
                    Group {
                        data: item.clone(),
                        index,
                    },
                );
            }
        }

        Ok(Value::Undefined)
    };

    if !input.is_array() {
        evaluate_group_item(input)?;
    } else if input.is_empty() {
        evaluate_group_item(&UNDEFINED)?;
    } else {
        for item in input.iter() {
            evaluate_group_item(item)?;
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

fn evaluate_filter(node: &Node, input: &Value, _frame: FrameLink) -> Result<Value> {
    let mut result = Value::Array {
        items: Vec::new(),
        is_sequence: true,
        cons: false,
        keep_singleton: false,
    };

    match node.kind {
        NodeKind::Filter(ref filter) => {
            match filter.kind {
                NodeKind::Number(n) => {
                    let mut index = n.floor() as isize;
                    let length = if input.is_array() {
                        input.len() as isize
                    } else {
                        1
                    };
                    if index < 0 {
                        // Count from the end of the array
                        index += length;
                    }
                    let item = if let Value::Array { items, .. } = input {
                        items.get(index as usize)
                    } else {
                        Some(input)
                    };
                    if let Some(item) = item {
                        if item.is_array() {
                            result = item.clone();
                        } else {
                            result.push(item.clone());
                        }
                    }
                }
                _ => unimplemented!("Filters other than numbers are not yet supported"),
            }
        }
        _ => unimplemented!("Filters other than numbers are not yet supported"),
    };

    Ok(result)
}

fn evaluate_path(node: &Node, steps: &[Node], input: &Value, frame: FrameLink) -> Result<Value> {
    // FIXME: How do I avoid these clones? It's so frustrating
    let mut input = if input.is_array() && !matches!(steps[0].kind, NodeKind::Var(..)) {
        input.clone()
    } else {
        Value::with_items(vec![input.clone()])
    };

    let mut result = Value::Undefined;

    for (index, step) in steps.iter().enumerate() {
        result = if index == 0 && step.cons_array {
            evaluate(step, &input, frame.clone())?
        } else {
            evaluate_step(step, &input, frame.clone(), index == steps.len() - 1)?
        };

        if result.is_undefined() || (result.is_array() && result.is_empty()) {
            break;
        }

        // if step.focus.is_none() {
        input = result.clone();
        // }
    }

    if node.keep_singleton_array {
        if let Value::Array {
            ref cons,
            ref mut keep_singleton,
            ref mut is_sequence,
            ..
        } = result
        {
            if *cons && !*is_sequence {
                *is_sequence = true;
            }
            *keep_singleton = true;
        } else {
            unreachable!("The result should be a sequence at this point")
        }
    }

    if let Some((position, ref object)) = node.group_by {
        result = evaluate_group_expression(position, object, &result, frame)?;
    }

    Ok(result)
}

fn evaluate_step(node: &Node, input: &Value, frame: FrameLink, last_step: bool) -> Result<Value> {
    let mut result = Value::Array {
        items: Vec::new(),
        is_sequence: true,
        cons: false,
        keep_singleton: false,
    };

    if let NodeKind::Sort(ref sorts) = node.kind {
        result = evaluate_sorts(sorts, input, frame.clone())?;
        if let Some(ref stages) = node.stages {
            result = evaluate_stages(stages, &result, frame)?;
        }
        return Ok(result);
    }

    for input in input.iter() {
        let mut input_result = evaluate(node, input, frame.clone())?;
        if let Some(ref stages) = node.stages {
            for stage in stages {
                input_result = evaluate_filter(stage, &input_result, frame.clone())?;
            }
        }
        if !input_result.is_undefined() {
            result.push(input_result);
        }
    }

    Ok(
        if last_step && result.len() == 1 && result[0].is_array() && !result[0].is_sequence() {
            std::mem::take(&mut result[0])
        } else {
            // Flatten the sequence
            let mut result_sequence = Value::Array {
                items: Vec::new(),
                is_sequence: true,
                cons: false,
                keep_singleton: false,
            };
            for result_item in result.iter_mut() {
                if !result_item.is_array() || result_item.cons() {
                    result_sequence.push(std::mem::take(result_item));
                } else {
                    for item in result_item.iter_mut() {
                        result_sequence.push(std::mem::take(item))
                    }
                }
            }

            result_sequence
        },
    )
}

fn evaluate_sorts(_sorts: &[(Node, bool)], _inputt: &Value, _frame: FrameLink) -> Result<Value> {
    unimplemented!("Sorts not yet implemented")
}

fn evaluate_stages(_stages: &[Node], _input: &Value, _frame: FrameLink) -> Result<Value> {
    unimplemented!("Stages not yet implemented")
}
