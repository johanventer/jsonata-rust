use json::JsonValue;
use std::collections::HashMap;
use std::rc::Rc;

use super::{Frame, FramePtr, Value};
use crate::error::*;
use crate::functions::*;
use crate::parser::ast::*;
use crate::Result;

pub(crate) fn evaluate(node: &Box<Node>, input: Rc<Value>, frame: FramePtr) -> Result<Rc<Value>> {
    let mut result = match node.kind {
        NodeKind::Path(ref steps) => evaluate_path(node, steps, input, Rc::clone(&frame))?,
        NodeKind::Binary(ref op, ref lhs, ref rhs) => {
            evaluate_binary_op(node, op, lhs, rhs, input, Rc::clone(&frame))?
        }
        NodeKind::Unary(ref op) => evaluate_unary_op(node, op, input, Rc::clone(&frame))?,
        NodeKind::Name(ref key) => lookup(input, key),
        NodeKind::Null => Rc::new(Value::Raw(JsonValue::Null)),
        NodeKind::Bool(ref value) => Rc::new(Value::Raw(json::from(*value))),
        NodeKind::Str(ref value) => Rc::new(Value::Raw(json::from(value.clone()))),
        NodeKind::Num(ref value) => Rc::new(Value::Raw(json::from(*value))),
        NodeKind::Ternary {
            ref cond,
            ref truthy,
            ref falsy,
        } => evaluate_ternary(cond, truthy, falsy.as_ref(), input, Rc::clone(&frame))?,
        NodeKind::Block(ref exprs) => evaluate_block(exprs, input, Rc::clone(&frame))?,
        NodeKind::Var(ref name) => evaluate_variable(name, input, Rc::clone(&frame))?,
        NodeKind::Wildcard => evaluate_wildcard(input)?,
        NodeKind::Descendent => evaluate_descendents(input)?,
        // NodeKind::Lambda { ref args, ref body } => {
        //     evaluate_lambda(args, body, input, Rc::clone(&frame))?
        // }
        //         NodeKind::Function {
        //             proc,
        //             args,
        //             is_partial,
        //         } => evaluate_function(proc, args, *is_partial, input, Rc::clone(&frame))?,
        //         // TODO:
        //         //  - Parent
        //         //  - Regex
        //         //  - Partial
        //         //  - Apply
        //         //  - Transform
        _ => unimplemented!("TODO: node kind not yet supported: {}", node.kind),
    };

    if let Some(ref predicates) = node.predicates {
        for predicate in predicates {
            result = evaluate_filter(predicate, result, Rc::clone(&frame))?;
        }
    }

    match &node.group_by {
        Some(object) if !node.is_path() => {
            result = evaluate_group_expression(node, object, result, Rc::clone(&frame))?;
        }
        _ => {}
    }

    if result.is_seq() {
        if node.keep_array {
            result.set_keep_singleton();
        }
        if result.arr().len() == 0 {
            Ok(Rc::new(Value::Undef))
        } else if result.arr().len() == 1 {
            if result.keep_singleton() {
                Ok(result)
            } else {
                Ok(Rc::clone(&result.arr()[0]))
            }
        } else {
            Ok(result)
        }
    } else {
        Ok(result)
    }
}

#[inline]
fn evaluate_unary_op(
    node: &Box<Node>,
    op: &UnaryOp,
    input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    match op {
        UnaryOp::Minus(ref v) => {
            let result = evaluate(v, input, frame)?;
            if result.is_undef() {
                Ok(Rc::new(Value::Undef))
            } else if let Some(num) = result.as_raw().as_f64() {
                Ok(Rc::new(Value::Raw((-num).into())))
            } else {
                Err(Box::new(D1002 {
                    position: node.position,
                    value: result.as_raw().to_string(),
                }))
            }
        }
        UnaryOp::ArrayConstructor(ref items) => {
            let mut result = Rc::new(Value::new_arr());
            for item in items {
                let value = evaluate(item, Rc::clone(&input), Rc::clone(&frame))?;
                if !value.is_undef() {
                    if matches!(item.kind, NodeKind::Unary(UnaryOp::ArrayConstructor(..))) {
                        result.arr_mut().push(value)
                    } else {
                        result = append(result, value);
                    }
                }
            }
            if node.cons_array {
                result.set_cons_array();
            }
            Ok(result)
        }
        UnaryOp::ObjectConstructor(object) => evaluate_group_expression(node, object, input, frame),
    }
}

fn evaluate_group_expression(
    node: &Box<Node>,
    object: &Object,
    mut input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    if !input.is_array() {
        input = Rc::new(Value::seq_from(input));
    }

    let mut groups: HashMap<String, (Rc<Value>, usize)> = HashMap::new();

    for input in input.arr().iter() {
        for (i, (k, _)) in object.iter().enumerate() {
            let key = evaluate(k, Rc::clone(input), Rc::clone(&frame))?;
            let key = key.as_raw().as_str();

            if key.is_none() {
                return Err(box T1003 {
                    position: node.position,
                    value: k.kind.to_string(),
                });
            }

            let key = key.unwrap();

            if groups.contains_key(key) {
                if groups[key].1 != i {
                    return Err(box D1009 {
                        position: node.position,
                        value: k.kind.to_string(),
                    });
                }

                groups.insert(
                    key.to_string(),
                    (append(Rc::clone(&groups[key].0), Rc::clone(input)), i),
                );
            } else {
                groups.insert(key.to_string(), (input.clone(), i));
            }
        }
    }

    let mut result = JsonValue::Object(json::object::Object::new());
    for key in groups.keys() {
        let value = evaluate(
            &object[groups[key].1].1,
            Rc::clone(&groups[key].0),
            Rc::clone(&frame),
        )?;
        if !value.is_undef() {
            result.insert(key, value.as_json()).unwrap();
        }
    }

    Ok(Rc::new(Value::Raw(result)))
}

#[inline]
fn evaluate_binary_op(
    node: &Box<Node>,
    op: &BinaryOp,
    lhs: &Box<Node>,
    rhs: &Box<Node>,
    input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    use BinaryOp::*;

    if *op == Bind {
        return evaluate_bind_expression(lhs, rhs, input, frame);
    }

    let lhs = evaluate(lhs, Rc::clone(&input), Rc::clone(&frame))?;
    let rhs = evaluate(rhs, input, frame)?;

    match op {
        Add | Subtract | Multiply | Divide | Modulus => {
            evaluate_numeric_expression(node, op, lhs, rhs)
        }
        LessThan | LessThanEqual | GreaterThan | GreaterThanEqual => {
            evaluate_comparison_expression(node, op, lhs, rhs)
        }
        Equal | NotEqual => evaluate_equality_expression(op, lhs, rhs),
        Concat => evaluate_string_concat(lhs, rhs),
        Or | And => evaluate_boolean_expression(op, lhs, rhs),
        In => evaluate_includes_expression(lhs, rhs),
        Range => evaluate_range_expression(node, lhs, rhs),
        _ => unreachable!("Unexpected binary operator {:#?}", op),
    }
}

#[inline]
fn evaluate_numeric_expression(
    node: &Box<Node>,
    op: &BinaryOp,
    lhs: Rc<Value>,
    rhs: Rc<Value>,
) -> Result<Rc<Value>> {
    if !lhs.is_raw() || !rhs.is_raw() {
        return Ok(Rc::new(Value::Undef));
    }

    let lhs: f64 = match lhs.as_raw() {
        JsonValue::Number(value) => value.clone().into(),
        _ => {
            return Err(Box::new(T2001 {
                position: node.position,
                op: op.to_string(),
            }))
        }
    };

    let rhs: f64 = match rhs.as_raw() {
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

    Ok(Rc::new(Value::Raw(result.into())))
}

#[inline]
fn evaluate_comparison_expression(
    node: &Box<Node>,
    op: &BinaryOp,
    lhs: Rc<Value>,
    rhs: Rc<Value>,
) -> Result<Rc<Value>> {
    if !lhs.is_raw() || !rhs.is_raw() {
        return Ok(Rc::new(Value::Undef));
    }

    let lhs = lhs.as_raw();
    let rhs = rhs.as_raw();

    if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
        return Err(Box::new(T2010 {
            position: node.position,
            op: op.to_string(),
        }));
    }

    if lhs.is_number() && rhs.is_number() {
        let lhs = lhs.as_f64().unwrap();
        let rhs = rhs.as_f64().unwrap();

        return Ok(Rc::new(Value::Raw(json::from(match op {
            BinaryOp::LessThan => lhs < rhs,
            BinaryOp::LessThanEqual => lhs <= rhs,
            BinaryOp::GreaterThan => lhs > rhs,
            BinaryOp::GreaterThanEqual => lhs >= rhs,
            _ => unreachable!(),
        }))));
    }

    if lhs.is_string() && rhs.is_string() {
        let lhs = lhs.as_str().unwrap();
        let rhs = rhs.as_str().unwrap();

        return Ok(Rc::new(Value::Raw(json::from(match op {
            BinaryOp::LessThan => lhs < rhs,
            BinaryOp::LessThanEqual => lhs <= rhs,
            BinaryOp::GreaterThan => lhs > rhs,
            BinaryOp::GreaterThanEqual => lhs >= rhs,
            _ => unreachable!(),
        }))));
    }

    Err(Box::new(T2009 {
        position: node.position,
        lhs: lhs.to_string(),
        rhs: rhs.to_string(),
        op: op.to_string(),
    }))
}

#[inline]
fn evaluate_equality_expression(
    op: &BinaryOp,
    lhs: Rc<Value>,
    rhs: Rc<Value>,
) -> Result<Rc<Value>> {
    if lhs.is_undef() || rhs.is_undef() {
        return Ok(Rc::new(Value::Raw(false.into())));
    }

    let result = match op {
        BinaryOp::Equal => lhs == rhs,
        BinaryOp::NotEqual => lhs != rhs,
        _ => unreachable!(),
    };

    Ok(Rc::new(Value::Raw(result.into())))
}

#[inline]
fn evaluate_string_concat(lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>> {
    let mut lstr = if lhs.is_undef() {
        "".to_owned()
    } else {
        string(lhs).unwrap()
    };

    let rstr = if rhs.is_undef() {
        "".to_owned()
    } else {
        string(rhs).unwrap()
    };

    lstr.push_str(&rstr);

    Ok(Rc::new(Value::Raw(lstr.into())))
}

#[inline]
fn evaluate_boolean_expression(op: &BinaryOp, lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>> {
    let left_bool = boolean(lhs);
    let right_bool = boolean(rhs);

    let result = match op {
        BinaryOp::And => left_bool && right_bool,
        BinaryOp::Or => left_bool || right_bool,
        _ => unreachable!(),
    };

    Ok(Rc::new(Value::Raw(result.into())))
}

#[inline]
fn evaluate_includes_expression(lhs: Rc<Value>, rhs: Rc<Value>) -> Result<Rc<Value>> {
    if lhs.is_undef() || rhs.is_undef() {
        return Ok(Rc::new(Value::Raw(false.into())));
    }

    if !rhs.is_array() {
        return Ok(Rc::new(Value::Raw((lhs.as_raw() == rhs.as_raw()).into())));
    }

    for item in rhs.arr().iter() {
        if item.is_raw() && lhs.as_raw() == item.as_raw() {
            return Ok(Rc::new(Value::Raw(true.into())));
        }
    }

    return Ok(Rc::new(Value::Raw(false.into())));
}

#[inline]
fn evaluate_range_expression(
    node: &Box<Node>,
    lhs: Rc<Value>,
    rhs: Rc<Value>,
) -> Result<Rc<Value>> {
    if lhs.is_undef() || rhs.is_undef() {
        return Ok(Rc::new(Value::Undef));
    }

    let lhs = match lhs.as_isize() {
        Some(num) => num,
        None => {
            return Err(box T2003 {
                position: node.position,
            })
        }
    };

    let rhs = match rhs.as_isize() {
        Some(num) => num,
        None => {
            return Err(box T2004 {
                position: node.position,
            })
        }
    };

    if lhs > rhs {
        return Ok(Rc::new(Value::Undef));
    }

    let size = rhs - lhs + 1;
    if size > 10_000_000_000 {
        return Err(box D2014 {
            position: node.position,
            value: size.to_string(),
        });
    }

    // TODO: This is quite slow with the max 10,000,000,000 items as there is a mem allocation for
    // each number
    let result = Rc::new(Value::seq_with_capacity(size as usize));
    for i in lhs..rhs + 1 {
        result.arr_mut().push(Rc::new(Value::Raw(i.into())))
    }

    Ok(result)
}

#[inline]
fn evaluate_bind_expression(
    lhs: &Box<Node>,
    rhs: &Box<Node>,
    input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    let rhs = evaluate(rhs, Rc::clone(&input), Rc::clone(&frame))?;

    if let NodeKind::Var(name) = &lhs.kind {
        frame.borrow_mut().bind(name, rhs)
    }

    Ok(input)
}

#[inline]
fn evaluate_ternary(
    cond: &Box<Node>,
    truthy: &Box<Node>,
    falsy: Option<&Box<Node>>,
    input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    let cond = evaluate(cond, Rc::clone(&input), Rc::clone(&frame))?;
    if boolean(cond) {
        evaluate(truthy, Rc::clone(&input), Rc::clone(&frame))
    } else if let Some(falsy) = falsy {
        evaluate(falsy, Rc::clone(&input), Rc::clone(&frame))
    } else {
        Ok(Rc::new(Value::Undef))
    }
}

#[inline]
fn evaluate_block(exprs: &Vec<Box<Node>>, input: Rc<Value>, frame: FramePtr) -> Result<Rc<Value>> {
    let frame = Frame::ptr_with_parent(frame);
    let mut result = Rc::new(Value::Undef);

    for expr in exprs {
        result = evaluate(expr, Rc::clone(&input), Rc::clone(&frame))?;
    }

    Ok(result)
}

#[inline]
fn evaluate_variable(name: &str, input: Rc<Value>, frame: FramePtr) -> Result<Rc<Value>> {
    if name == "" {
        // Empty variable name returns the context value
        if input.is_wrapped() {
            Ok(Rc::clone(&input.arr()[0]))
        } else {
            Ok(Rc::clone(&input))
        }
    } else {
        if let Some(value) = frame.borrow().lookup(name) {
            Ok(value)
        } else {
            Ok(Rc::new(Value::Undef))
        }
    }
}

#[inline]
fn evaluate_wildcard(input: Rc<Value>) -> Result<Rc<Value>> {
    let result = Rc::new(Value::new_seq());

    fn flatten(value: Rc<Value>, result: Rc<Value>) {
        if value.is_array() {
            value.arr().iter().for_each(|value| {
                flatten(Rc::clone(value), Rc::clone(&result));
            });
        } else {
            result.arr_mut().push(Rc::clone(&value));
        }
    }

    if input.as_raw().is_object() {
        for (_key, value) in input.as_raw().entries() {
            let value = Rc::new(Value::from_raw(Some(value)));
            if value.is_array() {
                flatten(Rc::clone(&value), Rc::clone(&result));
            } else {
                result.arr_mut().push(value);
            }
        }
    }

    Ok(result)
}

#[inline]
fn evaluate_descendents(input: Rc<Value>) -> Result<Rc<Value>> {
    let mut result = Rc::new(Value::Undef);
    let result_seq = Rc::new(Value::new_seq());

    fn recurse(value: Rc<Value>, result: Rc<Value>) {
        if !value.is_array() {
            result.arr_mut().push(Rc::clone(&value));
        }
        if value.is_array() {
            value
                .arr()
                .iter()
                .for_each(|value| recurse(Rc::clone(value), Rc::clone(&result)));
        } else if value.as_raw().is_object() {
            for (_key, value) in value.as_raw().entries() {
                let value = Rc::new(Value::from_raw(Some(value)));
                recurse(Rc::clone(&value), Rc::clone(&result));
            }
        }
    }

    if !input.is_undef() {
        recurse(Rc::clone(&input), Rc::clone(&result_seq));
        if result_seq.arr().len() == 1 {
            result = Rc::clone(&result_seq.arr()[0]);
        } else {
            result = result_seq;
        }
    }

    Ok(result)
}

#[inline]
fn evaluate_path(
    node: &Box<Node>,
    steps: &Vec<Box<Node>>,
    input: Rc<Value>,
    frame: FramePtr,
) -> Result<Rc<Value>> {
    let mut input = if !input.is_array() || matches!(&steps[0].kind, NodeKind::Var(..)) {
        Rc::new(Value::seq_from(input))
    } else {
        input
    };

    let mut result = Rc::new(Value::Undef);

    for (step_index, step) in steps.iter().enumerate() {
        // If the first step is an explicit array constructor, just evaluate it
        if step_index == 0 && step.cons_array {
            result = evaluate(step, Rc::clone(&input), Rc::clone(&frame))?;
        } else {
            result = evaluate_step(
                step,
                Rc::clone(&input),
                Rc::clone(&frame),
                step_index == steps.len() - 1,
            )?;
        }

        match *result {
            Value::Undef => break,
            Value::Array { .. } => {
                if result.arr().is_empty() {
                    break;
                }

                input = Rc::clone(&result);
            }
            _ => panic!("unexpected Value type"),
        }
    }

    if node.keep_singleton_array {
        if !result.is_seq() {
            result = Rc::new(Value::seq_from(result));
        }
        result.set_keep_singleton();
    }

    // TODO: Tuple, singleton array (jsonata.js:164)

    match &node.group_by {
        Some(object) => {
            result = evaluate_group_expression(node, object, result, Rc::clone(&frame))?;
        }
        _ => {}
    }

    Ok(result)
}

fn evaluate_step(
    node: &Box<Node>,
    input: Rc<Value>,
    frame: FramePtr,
    last_step: bool,
) -> Result<Rc<Value>> {
    let result = Rc::new(Value::new_seq());

    // if let NodeKind::Sort = node.kind {
    //     result = evaluate_sort_expression(node, input, frame);
    //     if node.stages.is_some() {
    //       result = evaluate_stages(node.stages, &result, frame)?;
    //     }
    // }

    for input in input.arr().iter() {
        let mut res = evaluate(node, Rc::clone(&input), Rc::clone(&frame))?;

        if let Some(ref stages) = node.stages {
            for stage in stages {
                res = evaluate_filter(stage, Rc::clone(&res), Rc::clone(&frame))?;
            }
        }

        if !res.is_undef() {
            result.arr_mut().push(res);
        }
    }

    if last_step
        && result.arr().len() == 1
        && result.arr()[0].is_array()
        && !result.arr()[0].is_seq()
    {
        Ok(Rc::clone(&result.arr()[0]))
    } else {
        // Flatten the result
        let flattened = Rc::new(Value::new_seq());
        result.arr().iter().for_each(|v| {
            if !v.is_array() || v.cons_array() {
                flattened.arr_mut().push(Rc::clone(v))
            } else {
                v.arr()
                    .iter()
                    .for_each(|v| flattened.arr_mut().push(Rc::clone(v)))
            }
        });
        Ok(flattened)
    }
}

fn evaluate_filter(node: &Box<Node>, mut input: Rc<Value>, frame: FramePtr) -> Result<Rc<Value>> {
    let mut results = Rc::new(Value::new_seq());

    if !input.is_array() {
        input = Rc::new(Value::seq_from(input));
    }

    if let NodeKind::Num(num) = node.kind {
        let index = if num < 0. {
            (num.floor() as isize).wrapping_add(input.arr().len() as isize) as usize
        } else {
            num.floor() as usize
        };

        if index < input.arr().len() {
            let item = &input.arr()[index as usize];
            if !item.is_undef() {
                if item.is_array() {
                    results = item.clone();
                } else {
                    results.arr_mut().push(Rc::clone(item));
                }
            }
        }
    } else {
        for (index, item) in input.arr().iter().enumerate() {
            let res = evaluate(node, Rc::clone(&item), Rc::clone(&frame))?;

            let indices = if let Some(num) = res.as_raw().as_f64() {
                vec![num]
            } else if let Some(indices) = res.as_f64_vec() {
                indices
            } else {
                vec![]
            };

            if !indices.is_empty() {
                indices.iter().for_each(|num| {
                    let ii = if *num < 0. {
                        (num.floor() as isize).wrapping_add(input.arr().len() as isize) as usize
                    } else {
                        num.floor() as usize
                    };
                    if ii == index {
                        results.arr_mut().push(Rc::clone(item));
                    }
                });
            } else if boolean(res) {
                results.arr_mut().push(Rc::clone(item));
            }
        }
    }

    Ok(results)
}
