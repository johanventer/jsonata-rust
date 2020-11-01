use crate::ast::BinaryOp::*;
use crate::ast::NodeKind::*;
use crate::ast::*;
use crate::error::*;
use crate::JsonAtaResult;
use json::JsonValue;

pub fn evaluate(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    match &node.kind {
        Null => Ok(Some(JsonValue::Null)),
        Bool(value) => Ok(Some(json::from(*value))),
        Str(value) => Ok(Some(json::from(value.clone()))),
        Num(value) => Ok(Some(json::from(*value))),
        Name(_) => evaluate_name(node, input),
        Unary(_) => evaluate_unary_op(node, input),
        Binary(_) => evaluate_binary_op(node, input),
        Block => evaluate_block(node, input),
        Ternary => evaluate_ternary(node, input),
        _ => unimplemented!("TODO: node kind not yet supported: {}", node.kind),
    }
}

fn evaluate_ternary(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let Ternary = &node.kind {
        let condition = evaluate(&node.children[0], input)?;
        if boolean(condition.as_ref()) {
            evaluate(&node.children[1], input)
        } else {
            evaluate(&node.children[2], input)
        }
    } else {
        unreachable!()
    }
}

fn evaluate_name(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let Name(value) = &node.kind {
        Ok(lookup(input, value))
    } else {
        unreachable!()
    }
}

fn evaluate_block(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let Block = &node.kind {
        // TODO: block frame
        let mut result: JsonAtaResult<Option<JsonValue>> = Ok(None);

        for child in &node.children {
            result = evaluate(child, input);
        }

        result
    } else {
        unreachable!();
    }
}

fn evaluate_unary_op(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let Unary(ref op) = &node.kind {
        match op {
            UnaryOp::Minus => {
                let value = evaluate(&node.children[0], input)?;
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
                    result.push(evaluate(child, input)?).unwrap();
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

fn evaluate_binary_op(node: &Node, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
    if let Binary(ref op) = &node.kind {
        let lhs = evaluate(&node.children[0], input)?;
        let rhs = evaluate(&node.children[1], input)?;
        match op {
            Add | Subtract | Multiply | Divide | Modulus => {
                evaluate_numeric_expression(node, lhs, rhs, op)
            }
            LessThan | LessThanEqual | GreaterThan | GreaterThanEqual => {
                evaluate_comparison_expression(node, lhs, rhs, op)
            }
            Equal | NotEqual => evaluate_equality_expression(lhs, rhs, op),
            Concat => evaluate_string_concat(lhs, rhs),
            _ => unimplemented!("TODO: Binary op {:?} not yet supported", op),
        }
    } else {
        unreachable!()
    }
}

fn evaluate_numeric_expression(
    node: &Node,
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
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
        Add => lhs + rhs,
        Subtract => lhs - rhs,
        Multiply => lhs * rhs,
        Divide => lhs / rhs,
        Modulus => lhs % rhs,
        _ => unreachable!(),
    };

    Ok(Some(result.into()))
}

fn evaluate_comparison_expression(
    node: &Node,
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
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
            LessThan => lhs < rhs,
            LessThanEqual => lhs <= rhs,
            GreaterThan => lhs > rhs,
            GreaterThanEqual => lhs >= rhs,
            _ => unreachable!(),
        })));
    }

    if lhs.is_string() && rhs.is_string() {
        let lhs = lhs.as_str().unwrap();
        let rhs = rhs.as_str().unwrap();

        return Ok(Some(json::from(match op {
            LessThan => lhs < rhs,
            LessThanEqual => lhs <= rhs,
            GreaterThan => lhs > rhs,
            GreaterThanEqual => lhs >= rhs,
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

fn evaluate_equality_expression(
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> JsonAtaResult<Option<JsonValue>> {
    if lhs.is_none() && rhs.is_none() {
        return Ok(Some(true.into()));
    }

    let lhs = lhs.unwrap();
    let rhs = rhs.unwrap();

    let result = match op {
        Equal => lhs == rhs,
        NotEqual => lhs != rhs,
        _ => unreachable!(),
    };

    Ok(Some(result.into()))
}

fn evaluate_string_concat(
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
) -> JsonAtaResult<Option<JsonValue>> {
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
