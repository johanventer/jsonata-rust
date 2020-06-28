use crate::ast::BinaryOp::*;
use crate::ast::NodeKind::*;
use crate::ast::*;
use crate::error::EvaluatorError;
use crate::error::EvaluatorError::*;
use json::JsonValue;

type Result = std::result::Result<Option<JsonValue>, EvaluatorError>;

pub fn evaluate(node: &Node) -> Result {
    match &node.kind {
        Null => Ok(Some(JsonValue::Null)),
        Bool(value) => Ok(Some(json::from(*value))),
        Str(value) => Ok(Some(json::from(value.clone()))),
        Num(value) => Ok(Some(json::from(*value))),
        Binary(ref op) => evaluate_binary_expression(node),
        _ => Ok(None),
    }
}

fn evaluate_binary_expression(node: &Node) -> Result {
    if let Binary(ref op) = &node.kind {
        let lhs = evaluate(&node.children[0])?;
        let rhs = evaluate(&node.children[1])?;
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
            _ => return Err(LeftSideMustBeNumber),
        },
        None => return Ok(None),
    };

    let rhs: f64 = match rhs {
        Some(value) => match value {
            JsonValue::Number(value) => value.into(),
            _ => return Err(RightSideMustBeNumber),
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

    Err(InvalidComparison)
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
