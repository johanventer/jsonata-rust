use jsonata_errors::Error;
use jsonata_shared::Position;

use crate::ast::BinaryOp;
use crate::functions::FunctionContext;
use crate::tokenizer::TokenKind;
use crate::value::Value;

pub fn i0205_wrong_type(expected: &str) -> Error {
    Error::I0205WrongType(expected.into())
}

pub fn s0202_unexpected_token(p: Position, e: &TokenKind, a: &TokenKind) -> Error {
    Error::S0202UnexpectedToken(p, e.to_string(), a.to_string())
}

pub fn s0203_expected_token_before_end(p: Position, k: &TokenKind) -> Error {
    Error::S0203ExpectedTokenBeforeEnd(p, k.to_string())
}

pub fn s0211_invalid_unary(p: Position, k: &TokenKind) -> Error {
    Error::S0211InvalidUnary(p, k.to_string())
}

pub fn s0213_invalid_step(p: Position, k: &str) -> Error {
    Error::S0213InvalidStep(p, k.to_string())
}

pub fn s0208_invalid_function_param(p: Position, k: &TokenKind) -> Error {
    Error::S0208InvalidFunctionParam(p, k.to_string())
}

pub fn s0214_expected_var_right(p: Position, k: &str) -> Error {
    Error::S0214ExpectedVarRight(p, k.to_string())
}

pub fn d1002_negating_non_numeric(p: Position, v: &Value) -> Error {
    Error::D1002NegatingNonNumeric(p, format!("{}", v))
}

pub fn t1003_non_string_key(p: Position, v: &Value) -> Error {
    Error::T1003NonStringKey(p, format!("{}", v))
}

pub fn d1009_multiple_keys(p: Position, k: &str) -> Error {
    Error::D1009MultipleKeys(p, k.to_string())
}

pub fn t2001_left_side_not_number(p: Position, o: &BinaryOp) -> Error {
    Error::T2001LeftSideNotNumber(p, o.to_string())
}

pub fn t2002_right_side_not_number(p: Position, o: &BinaryOp) -> Error {
    Error::T2002RightSideNotNumber(p, o.to_string())
}

pub fn t2009_binary_op_mismatch(p: Position, l: &Value, r: &Value, o: &BinaryOp) -> Error {
    Error::T2009BinaryOpMismatch(p, format!("{}", l), format!("{}", r), o.to_string())
}

pub fn t2010_binary_op_types(p: Position, o: &BinaryOp) -> Error {
    Error::T2010BinaryOpTypes(p, o.to_string())
}

pub fn t0410_argument_not_valid(context: &FunctionContext, arg_index: usize) -> Error {
    Error::T0410ArgumentNotValid(context.position, arg_index, context.name.to_string())
}

pub fn t0412_argument_must_be_array_of_type(
    context: &FunctionContext,
    arg_index: usize,
    ty: &str,
) -> Error {
    Error::T0412ArgumentMustBeArrayOfType(
        context.position,
        arg_index,
        context.name.to_string(),
        ty.to_string(),
    )
}
