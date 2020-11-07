use crate::ast::*;
use crate::error::*;
use crate::frame::{Binding, Frame};
use crate::JsonAtaResult;
use json::{array, JsonValue};

pub fn evaluate(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    let result = match &node.kind {
        NodeKind::Null => Some(JsonValue::Null),
        NodeKind::Bool(ref value) => Some(json::from(*value)),
        NodeKind::Str(ref value) => Some(json::from(value.clone())),
        NodeKind::Num(ref value) => Some(json::from(*value)),
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

    if let Some(mut result) = result {
        if result.is_array() {
            // TODO: Keep singleton (jsonata.js:143)

            if result.len() == 0 {
                return Ok(None);
            }

            if result.len() == 1 {
                return Ok(Some(result[0].take()));
            }
        }

        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn evaluate_path(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    match input {
        None => Ok(None),
        Some(mut input) => {
            // TODO: Tuple, singleton array, group expressions (jsonata.js:164)

            let mut result: Option<JsonValue> = None;

            for (step_index, step) in node.children.iter().enumerate() {
                result = evaluate_step(step, input, frame, step_index == input.len() - 1)?;

                match result {
                    None => break,
                    Some(ref result) => {
                        if result.is_empty() {
                            break;
                        } else {
                            input = result;
                        }
                    }
                }
            }

            Ok(result)
        }
    }
}

fn evaluate_step(
    node: &Node,
    input: &JsonValue,
    frame: &mut Frame,
    last_step: bool,
) -> JsonAtaResult<Option<JsonValue>> {
    // TODO: Sorting (jsonata.js:253)

    let mut result = array![];

    let mut evaluate_input = |input: &JsonValue| -> JsonAtaResult<Option<JsonValue>> {
        let input_result = evaluate(node, Some(input), frame)?;

        // TODO: Filtering (jsonata.js:267)

        if let Some(input_result) = input_result {
            result.push(input_result).unwrap();
        }

        Ok(None)
    };

    if input.is_array() {
        for input in input.members() {
            evaluate_input(input)?;
        }
    } else {
        evaluate_input(input)?;
    }

    return if last_step && result.len() == 1 && result[0].is_array() {
        Ok(Some(result[0].clone()))
    } else {
        // Flatten the sequence
        let mut flat_result = array![];
        result.members().for_each(|member| {
            if !member.is_array() {
                flat_result.push(member.clone()).unwrap();
            } else {
                member
                    .members()
                    .for_each(|member| flat_result.push(member.clone()).unwrap());
            }
        });
        Ok(Some(flat_result))
    };
}

fn evaluate_variable(name: &str, frame: &Frame) -> JsonAtaResult<Option<JsonValue>> {
    // TODO: Something special happens when value == ""
    if let Some(binding) = frame.lookup(name) {
        // TODO: I don't like this clone
        Ok(Some(binding.as_var().clone()))
    } else {
        Ok(None)
    }
}

fn evaluate_ternary(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    if let NodeKind::Ternary = &node.kind {
        let condition = evaluate(&node.children[0], input, frame)?;
        if boolean(condition.as_ref()) {
            evaluate(&node.children[1], input, frame)
        } else {
            evaluate(&node.children[2], input, frame)
        }
    } else {
        unreachable!()
    }
}

fn evaluate_name(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let NodeKind::Name(value) = &node.kind {
        Ok(lookup(input, value))
    } else {
        unreachable!()
    }
}

fn evaluate_block(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    if let NodeKind::Block = &node.kind {
        // TODO: block frame
        let mut result: JsonAtaResult<Option<JsonValue>> = Ok(None);

        for child in &node.children {
            result = evaluate(child, input, frame);
        }

        result
    } else {
        unreachable!();
    }
}

fn evaluate_unary_op(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    if let NodeKind::Unary(ref op) = &node.kind {
        match op {
            UnaryOp::Minus => {
                let value = evaluate(&node.children[0], input, frame)?;
                match value {
                    Some(value) => {
                        if let Some(value) = value.as_f64() {
                            Ok(Some((-value).into()))
                        } else {
                            Err(Box::new(D1002 {
                                position: node.position,
                                value: value.to_string(),
                            }))
                        }
                    }
                    None => Ok(None),
                }
            }
            UnaryOp::Array => {
                let mut result = JsonValue::new_array();
                for child in &node.children {
                    result.push(evaluate(child, input, frame)?).unwrap();
                }
                Ok(Some(result))
                // TODO: consarray
            }
            UnaryOp::Object => unimplemented!("TODO: object constructors not yet supported"),
        }
    } else {
        unreachable!();
    }
}

fn evaluate_binary_op(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
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
        unreachable!()
    }
}

fn evaluate_bind_expression(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    let name = &node.children[0];
    let value = evaluate(&node.children[1], input, frame)?.unwrap();
    if let NodeKind::Var(name) = &name.kind {
        frame.bind(name, Binding::Var(value));
    }
    Ok(None)
}

fn evaluate_numeric_expression(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let lhs: f64 = match lhs {
        Some(value) => match value {
            JsonValue::Number(value) => value.into(),
            _ => {
                return Err(Box::new(T2001 {
                    position: node.position,
                    op: op.to_string(),
                }))
            }
        },
        None => return Ok(None),
    };

    let rhs: f64 = match rhs {
        Some(value) => match value {
            JsonValue::Number(value) => value.into(),
            _ => {
                return Err(Box::new(T2002 {
                    position: node.position,
                    op: op.to_string(),
                }))
            }
        },
        None => return Ok(None),
    };

    let result = match op {
        BinaryOp::Add => lhs + rhs,
        BinaryOp::Subtract => lhs - rhs,
        BinaryOp::Multiply => lhs * rhs,
        BinaryOp::Divide => lhs / rhs,
        BinaryOp::Modulus => lhs % rhs,
        _ => unreachable!(),
    };

    Ok(Some(result.into()))
}

fn evaluate_comparison_expression(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let lhs = match lhs {
        Some(value) => value,
        None => return Ok(None),
    };

    let rhs = match rhs {
        Some(value) => value,
        None => return Ok(None),
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

        return Ok(Some(json::from(match op {
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

        return Ok(Some(json::from(match op {
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
    input: Option<&JsonValue>,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    let left_bool = boolean(lhs.as_ref());
    let right_bool = boolean(rhs.as_ref());

    let result = match op {
        BinaryOp::And => left_bool && right_bool,
        BinaryOp::Or => left_bool || right_bool,
        _ => unreachable!(),
    };

    Ok(Some(result.into()))
}

fn evaluate_includes_expression(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    if let Some(lhs) = lhs {
        if let Some(rhs) = rhs {
            if !rhs.is_array() {
                return Ok(Some((lhs == rhs).into()));
            }

            for item in rhs.members() {
                if &lhs == item {
                    return Ok(Some(true.into()));
                }
            }
        }
    } else {
        return Ok(Some(false.into()));
    }

    Ok(Some(false.into()))
}

fn evaluate_equality_expression(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    if lhs.is_none() && rhs.is_none() {
        return Ok(Some(true.into()));
    }

    let lhs = lhs.unwrap();
    let rhs = rhs.unwrap();

    let result = match op {
        BinaryOp::Equal => lhs == rhs,
        BinaryOp::NotEqual => lhs != rhs,
        _ => unreachable!(),
    };

    Ok(Some(result.into()))
}

fn evaluate_string_concat(
    node: &Node,
    input: Option<&JsonValue>,
    frame: &mut Frame,
) -> JsonAtaResult<Option<JsonValue>> {
    let lhs = evaluate(&node.children[0], input, frame)?;
    let rhs = evaluate(&node.children[1], input, frame)?;

    // TODO: FIXME: This needs lots of work, jsonata has some automatic stringification rules which need
    // implementing, so this will fail if you don't provide two JsonValue::Strings. Also, there's
    // too much string copying going on.
    let lhs = match lhs {
        Some(value) => value.as_str().unwrap().to_owned(),
        None => "".to_string(),
    };
    let rhs = match rhs {
        Some(value) => value.as_str().unwrap().to_owned(),
        None => "".to_string(),
    };
    let result = lhs + &rhs;
    Ok(Some(result.into()))
}

fn lookup(input: Option<&JsonValue>, key: &str) -> Option<JsonValue> {
    match input {
        Some(input) => {
            if input.is_array() {
                // TODO
                None
            } else if input.is_object() && input.has_key(key) {
                Some(input[key].clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn boolean(arg: Option<&JsonValue>) -> bool {
    match arg {
        None => false,
        Some(arg) => match arg {
            JsonValue::Null => false,
            JsonValue::Short(ref arg) => !arg.is_empty(),
            JsonValue::String(ref arg) => !arg.is_empty(),
            JsonValue::Number(ref arg) => !arg.is_zero(),
            JsonValue::Boolean(ref arg) => *arg,
            JsonValue::Object(ref arg) => !arg.is_empty(),
            JsonValue::Array(ref arg) => match arg.len() {
                0 => false,
                1 => boolean(Some(&arg[0])),
                _ => {
                    let trues: Vec<_> = arg.iter().filter(|x| boolean(Some(&x))).collect();
                    !trues.is_empty()
                }
            },
        },
    }
}
