use crate::ast::BinaryOp::*;
use crate::ast::NodeKind::*;
use crate::ast::*;
use crate::error::EvaluatorError;
use crate::error::EvaluatorError::*;
use json::JsonValue;

pub type Result = std::result::Result<Option<JsonValue>, EvaluatorError>;

pub fn evaluate(node: &Node, input: &JsonValue) -> Result {
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
        _ => Ok(None),
    }
}

fn evaluate_ternary(node: &Node, input: &JsonValue) -> Result {
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

fn evaluate_name(node: &Node, input: &JsonValue) -> Result {
    if let Name(value) = &node.kind {
        Ok(lookup(input, value))
    } else {
        unreachable!()
    }
}

fn evaluate_block(node: &Node, input: &JsonValue) -> Result {
    if let Block = &node.kind {
        // TODO: block frame
        let mut result: Result = Ok(None);

        for child in &node.children {
            result = evaluate(child, input);
        }

        result
    } else {
        unreachable!();
    }
}

fn evaluate_unary_op(node: &Node, input: &JsonValue) -> Result {
    if let Unary(ref op) = &node.kind {
        match op {
            UnaryOp::Minus => {
                let value = evaluate(&node.children[0], input)?;
                match value {
                    Some(value) => {
                        if let Some(value) = value.as_f64() {
                            Ok(Some((-value).into()))
                        } else {
                            Err(NonNumericNegation(value))
                        }
                    }
                    None => Ok(None),
                }
            }
            UnaryOp::Array => {
                let mut result = JsonValue::new_array();
                for child in &node.children {
                    result.push(evaluate(child, input)?)?;
                }
                Ok(Some(result))
                // TODO: consarray
            }
            UnaryOp::Object => {
                // TODO
                Ok(None)
            }
        }
    } else {
        unreachable!();
    }
}

fn evaluate_binary_op(node: &Node, input: &JsonValue) -> Result {
    if let Binary(ref op) = &node.kind {
        let lhs = evaluate(&node.children[0], input)?;
        let rhs = evaluate(&node.children[1], input)?;
        match op {
            Add | Subtract | Multiply | Divide | Modulus => {
                evaluate_numeric_expression(lhs, rhs, op)
            }
            LessThan | LessThanEqual | GreaterThan | GreaterThanEqual => {
                evaluate_comparison_expression(lhs, rhs, op)
            }
            Equal | NotEqual => evaluate_equality_expression(lhs, rhs, op),
            Concat => evaluate_string_concat(lhs, rhs),
            _ => Ok(None),
        }
    } else {
        unreachable!()
    }
}

fn evaluate_numeric_expression(
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> Result {
    let lhs: f64 = match lhs {
        Some(value) => match value {
            JsonValue::Number(value) => value.into(),
            _ => return Err(LeftSideMustBeNumber(op.clone())),
        },
        None => return Ok(None),
    };

    let rhs: f64 = match rhs {
        Some(value) => match value {
            JsonValue::Number(value) => value.into(),
            _ => return Err(RightSideMustBeNumber(op.clone())),
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
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> Result {
    let lhs = match lhs {
        Some(value) => value,
        None => return Ok(None),
    };

    let rhs = match rhs {
        Some(value) => value,
        None => return Ok(None),
    };

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

    Err(InvalidComparison(op.clone()))
}

fn evaluate_equality_expression(
    lhs: Option<JsonValue>,
    rhs: Option<JsonValue>,
    op: &BinaryOp,
) -> Result {
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

fn evaluate_string_concat(lhs: Option<JsonValue>, rhs: Option<JsonValue>) -> Result {
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

fn lookup(input: &JsonValue, key: &str) -> Option<JsonValue> {
    if input.is_array() {
        // TODO
        return None;
    } else if input.is_object() && input.has_key(key) {
        return Some(input[key].clone());
    }
    None
}

fn boolean(arg: Option<&JsonValue>) -> bool {
    match arg {
        None => false,
        Some(arg) => match arg {
            JsonValue::Null => false,
            JsonValue::Short(ref arg) => arg.len() > 0,
            JsonValue::String(ref arg) => arg.len() > 0,
            JsonValue::Number(ref arg) => !arg.is_zero(),
            JsonValue::Boolean(ref arg) => *arg,
            JsonValue::Object(ref arg) => arg.len() > 0,
            JsonValue::Array(ref arg) => match arg.len() {
                0 => false,
                1 => boolean(Some(&arg[0])),
                _ => {
                    let trues: Vec<_> = arg.into_iter().filter(|x| boolean(Some(&x))).collect();
                    trues.len() > 0
                }
            },
        },
    }
}
