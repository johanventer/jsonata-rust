use jsonata_errors::Error;
use jsonata_shared::Position;

use crate::ast::BinaryOp;
use crate::functions::FunctionContext;
use crate::tokenizer::TokenKind;
use crate::value::Value;

pub fn wrong_type(expected: &str) -> Error {
    Error::WrongType(expected.into())
}

pub fn syntax_error(p: Position, t: &TokenKind) -> Error {
    Error::SyntaxError(p, t.to_string())
}

pub fn unexpected_token(p: Position, e: &TokenKind, a: &TokenKind) -> Error {
    Error::UnexpectedToken(p, e.to_string(), a.to_string())
}

pub fn expected_token_before_end(p: Position, k: &TokenKind) -> Error {
    Error::ExpectedTokenBeforeEnd(p, k.to_string())
}

pub fn invalid_unary(p: Position, k: &TokenKind) -> Error {
    Error::InvalidUnary(p, k.to_string())
}

pub fn invalid_step(p: Position, k: &str) -> Error {
    Error::InvalidStep(p, k.to_string())
}

pub fn invalid_function_param(p: Position, k: &TokenKind) -> Error {
    Error::InvalidFunctionParam(p, k.to_string())
}

pub fn expected_var_right(p: Position, k: &str) -> Error {
    Error::ExpectedVarRight(p, k.to_string())
}

pub fn unsupported_escape(p: Position, c: char) -> Error {
    Error::UnsupportedEscape(p, c.to_string())
}

pub fn negating_non_numeric(p: Position, v: &Value) -> Error {
    Error::NegatingNonNumeric(p, format!("{}", v))
}

pub fn non_string_key(p: Position, v: &Value) -> Error {
    Error::NonStringKey(p, format!("{}", v))
}

pub fn multiple_keys(p: Position, k: &str) -> Error {
    Error::MultipleKeys(p, k.to_string())
}

pub fn left_side_not_number(p: Position, o: &BinaryOp) -> Error {
    Error::LeftSideNotNumber(p, o.to_string())
}

pub fn right_side_not_number(p: Position, o: &BinaryOp) -> Error {
    Error::RightSideNotNumber(p, o.to_string())
}

pub fn binary_op_mismatch(p: Position, l: &Value, r: &Value, o: &BinaryOp) -> Error {
    Error::BinaryOpMismatch(p, format!("{}", l), format!("{}", r), o.to_string())
}

pub fn binary_op_types(p: Position, o: &BinaryOp) -> Error {
    Error::BinaryOpTypes(p, o.to_string())
}

pub fn argument_not_valid(context: &FunctionContext, arg_index: usize) -> Error {
    Error::ArgumentNotValid(context.position, arg_index, context.name.to_string())
}

pub fn argument_must_be_array_of_type(
    context: &FunctionContext,
    arg_index: usize,
    ty: &str,
) -> Error {
    Error::ArgumentMustBeArrayOfType(
        context.position,
        arg_index,
        context.name.to_string(),
        ty.to_string(),
    )
}
