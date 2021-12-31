// use crate::parser::Position;

// pub trait Error: std::error::Error + std::fmt::Debug + std::fmt::Display {
//     fn code(&self) -> &str;
// }

// macro_rules! define_error {
//     ($name:ident, $template:literal, $( $arg:ident ),*) => {
//         pub struct $name {
//             pub position: Position,
//             $( pub $arg: String, )*
//         }

//         impl std::error::Error for $name {}

//         impl Error for $name {
//             fn code(&self) -> &str {
//                 stringify!($name)
//             }
//         }

//         impl std::fmt::Display for $name {
//             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 write!(f, concat!("Error @ character {}: {} - ", $template), self.position.source_pos, self.code(), $( self.$arg, )*)
//             }
//         }

//         impl std::fmt::Debug for $name {
//             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 write!(f, concat!("Error @ character {}: {} - ", $template), self.position.source_pos, self.code(), $( self.$arg, )*)
//             }
//         }
//     };

//     ($name:ident, $template:literal) => {
//         define_error!($name, $template,);
//     };
// }

// define_error!(InvalidJson, "The input is not valid JSON");
// define_error!(S0202, "Expected `{}`, got `{}`", expected, actual);
// define_error!(S0203, "Expected `{}` before end of expression", expected);
//define_error!(S0204, "Unknown operator: `{}`", token);
// "S0205": "Unexpected token: {{token}}",
// "S0206": "Unknown expression type: {{token}}",
// "S0207": "Unexpected end of expression",
// "S0215": "A context variable binding must precede any predicates on a step",
// "S0216": "A context variable binding must precede the 'order-by' clause on a step",
// "S0217": "The object representing the 'parent' cannot be derived from this expression",

// "S0301": "Empty regular expressions are not allowed",
// "S0302": "No terminating / in regular expression",
// "S0402": "Choice groups containing parameterized types are not supported",
// "S0401": "Type parameters can only be applied to functions and arrays",
// "S0500": "Attempted to evaluate an expression containing syntax error(s)",
// "T0410": "Argument {{index}} of function {{token}} does not match function signature",
// "T0411": "Context value is not a compatible type with argument {{index}} of function {{token}}",
// "T0412": "Argument {{index}} of function {{token}} must be an array of {{type}}",
// "D1001": "Number out of range: {{value}}",
// define_error!(D1002, "Cannot negate a non-numeric value `{}`", value);
// define_error!(
//     T1003,
//     "Key in object structure must evaluate to a string; got: {}",
//     value
// );
// "D1004": "Regular expression matches zero length string",
// "T1005": "Attempted to invoke a non-function. Did you mean ${{{token}}}?",
// "T1006": "Attempted to invoke a non-function",
// "T1007": "Attempted to partially apply a non-function. Did you mean ${{{token}}}?",
// "T1008": "Attempted to partially apply a non-function",
// define_error!(
//     D1009,
//     "Multiple key definitions evaluate to same key: {}",
//     value
// );
// // "T1010": "The matcher function argument passed to function {{token}} does not return the correct object structure",
// define_error!(
//     T2001,
//     "The left side of the `{}` operator must evaluate to a number",
//     op
// );
// define_error!(
//     T2002,
//     "The right side of the `{}` operator must evaluate to a number",
//     op
// );
// define_error!(
//     T2003,
//     "The left side of the range operator (..) must evaluate to an integer"
// );
// define_error!(
//     T2004,
//     "The right side of the range operator (..) must evaluate to an integer"
// );
// "D2005": "The left side of := must be a variable name (start with $)",  // defunct - replaced by S0212 parser error
// "T2006": "The right side of the function application operator ~> must be a function",
// "T2007": "Type mismatch when comparing values {{value}} and {{value2}} in order-by clause",
// "T2008": "The expressions within an order-by clause must evaluate to numeric or string values",
// define_error!(
//     T2009,
//     "The values {} and {} either side of operator {} must be of the same data type",
//     lhs,
//     rhs,
//     op
// );
// define_error!(
//     T2010,
//     "The expressions either side of operator `{}` must evaluate to numeric or string values",
//     op
// );
// "T2011": "The insert/update clause of the transform expression must evaluate to an object: {{value}}",
// "T2012": "The delete clause of the transform expression must evaluate to a string or array of strings: {{value}}",
// "T2013": "The transform expression clones the input object using the $clone() function.  This has been overridden in the current scope by a non-function.",
// define_error!(
//     D2014,
//     "The size of the sequence allocated by the range operator (..) must not exceed 1e7.  Attempted to allocate {}",
//     value
// );
// "D3001": "Attempting to invoke string function on Infinity or NaN",
// "D3010": "Second argument of replace function cannot be an empty string",
// "D3011": "Fourth argument of replace function must evaluate to a positive number",
// "D3012": "Attempted to replace a matched string with a non-string value",
// "D3020": "Third argument of split function must evaluate to a positive number",
// "D3030": "Unable to cast value to a number: {{value}}",
// "D3040": "Third argument of match function must evaluate to a positive number",
// "D3050": "The second argument of reduce function must be a function with at least two arguments",
// "D3060": "The sqrt function cannot be applied to a negative number: {{value}}",
// "D3061": "The power function has resulted in a value that cannot be represented as a JSON number: base={{value}}, exponent={{exp}}",
// "D3070": "The single argument form of the sort function can only be applied to an array of strings or an array of numbers.  Use the second argument to specify a comparison function",
// "D3080": "The picture string must only contain a maximum of two sub-pictures",
// "D3081": "The sub-picture must not contain more than one instance of the 'decimal-separator' character",
// "D3082": "The sub-picture must not contain more than one instance of the 'percent' character",
// "D3083": "The sub-picture must not contain more than one instance of the 'per-mille' character",
// "D3084": "The sub-picture must not contain both a 'percent' and a 'per-mille' character",
// "D3085": "The mantissa part of a sub-picture must contain at least one character that is either an 'optional digit character' or a member of the 'decimal digit family'",
// "D3086": "The sub-picture must not contain a passive character that is preceded by an active character and that is followed by another active character",
// "D3087": "The sub-picture must not contain a 'grouping-separator' character that appears adjacent to a 'decimal-separator' character",
// "D3088": "The sub-picture must not contain a 'grouping-separator' at the end of the integer part",
// "D3089": "The sub-picture must not contain two adjacent instances of the 'grouping-separator' character",
// "D3090": "The integer part of the sub-picture must not contain a member of the 'decimal digit family' that is followed by an instance of the 'optional digit character'",
// "D3091": "The fractional part of the sub-picture must not contain an instance of the 'optional digit character' that is followed by a member of the 'decimal digit family'",
// "D3092": "A sub-picture that contains a 'percent' or 'per-mille' character must not contain a character treated as an 'exponent-separator'",
// "D3093": "The exponent part of the sub-picture must comprise only of one or more characters that are members of the 'decimal digit family'",
// "D3100": "The radix of the formatBase function must be between 2 and 36.  It was given {{value}}",
// "D3110": "The argument of the toMillis function must be an ISO 8601 formatted timestamp. Given {{value}}",
// "D3120": "Syntax error in expression passed to function eval: {{value}}",
// "D3121": "Dynamic error evaluating the expression passed to function eval: {{value}}",
// "D3130": "Formatting or parsing an integer as a sequence starting with {{value}} is not supported by this implementation",
// "D3131": "In a decimal digit pattern, all digits must be from the same decimal group",
// "D3132": "Unknown component specifier {{value}} in date/time picture string",
// "D3133": "The 'name' modifier can only be applied to months and days in the date/time picture string, not {{value}}",
// "D3134": "The timezone integer format specifier cannot have more than four digits",
// "D3135": "No matching closing bracket ']' in date/time picture string",
// "D3136": "The date/time picture string is missing specifiers required to parse the timestamp",
// "D3137": "{{{message}}}",
// "D3138": "The $single() function expected exactly 1 matching result.  Instead it matched more.",
// "D3139": "The $single() function expected exactly 1 matching result.  Instead it matched 0.",
// "D3140": "Malformed URL passed to ${{{functionName}}}(): {{value}}",
// "D3141": "{{{message}}}"

use super::position::Position;
use super::tokenizer::TokenKind;
use std::{char, error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    // JSON parsing errors
    UnexpectedCharacter {
        //I01
        ch: char,
        line: usize,
        column: usize,
    },
    UnexpectedEndOfJson, // I02
    ExceededDepthLimit,  // I03
    FailedUtf8Parsing,   // I04
    WrongType(String),   // I05

    // Lexing errors
    UnterminatedStringLiteral(Position), // S0101
    NumberOutOfRange(Position, String),  // S0102
    UnsupportedEscape(Position, String), // S0103
    InvalidUnicodeEscape(Position),      // S0104
    UnterminatedQuoteProp(Position),     // S0105
    UnterminatedComment(Position),       // S0106

    // Parsing errors
    SyntaxError(Position, String),             // S0201
    UnexpectedToken(Position, String, String), // S0202
    ExpectedTokenBeforeEnd(Position, String),  // S0203
    InvalidFunctionParam(Position, String),    // S0208
    InvalidPredicate(Position),                // S0209
    MultipleGroupBy(Position),                 // S0210
    InvalidUnary(Position, String),            // S0211
    ExpectedVarLeft(Position),                 // S0212
    InvalidStep(Position, String),             // S0213
    ExpectedVarRight(Position, String),        // S0214
}

impl error::Error for Error {}

impl Error {
    pub fn code(&self) -> &str {
        match *self {
            // JSON parsing errors
            Error::UnexpectedCharacter { .. } => "I01",
            Error::UnexpectedEndOfJson => "I02",
            Error::ExceededDepthLimit => "I03",
            Error::FailedUtf8Parsing => "I04",
            Error::WrongType(_) => "I05",

            // Lexing errors
            Error::UnterminatedStringLiteral(_) => "S0101",
            Error::NumberOutOfRange(_, _) => "S0102",
            Error::UnsupportedEscape(_, _) => "S0103",
            Error::InvalidUnicodeEscape(_) => "S0104",
            Error::UnterminatedQuoteProp(_) => "S0105",
            Error::UnterminatedComment(_) => "S0106",

            // Parsing errors
            Error::SyntaxError(_, _) => "S0201",
            Error::UnexpectedToken(_, _, _) => "S0202",
            Error::ExpectedTokenBeforeEnd(_, _) => "S0203",
            Error::InvalidFunctionParam(_, _) => "S0208",
            Error::InvalidPredicate(_) => "S0209",
            Error::MultipleGroupBy(_) => "S0210",
            Error::InvalidUnary(_, _) => "S0211",
            Error::ExpectedVarLeft(_) => "S0212",
            Error::InvalidStep(_, _) => "S0213",
            Error::ExpectedVarRight(_, _) => "S0214",
        }
    }

    pub fn wrong_type(expected: &str) -> Self {
        Error::WrongType(expected.into())
    }

    pub fn syntax_error(p: Position, t: &TokenKind) -> Self {
        Error::SyntaxError(p, t.to_string())
    }

    pub fn unexpected_token(p: Position, e: &TokenKind, a: &TokenKind) -> Self {
        Error::UnexpectedToken(p, e.to_string(), a.to_string())
    }

    pub fn expected_token_before_end(p: Position, k: &TokenKind) -> Self {
        Error::ExpectedTokenBeforeEnd(p, k.to_string())
    }

    pub fn invalid_unary(p: Position, k: &TokenKind) -> Self {
        Error::InvalidUnary(p, k.to_string())
    }

    pub fn invalid_step(p: Position, k: &str) -> Self {
        Error::InvalidStep(p, k.to_string())
    }

    pub fn invalid_function_param(p: Position, k: &TokenKind) -> Self {
        Error::InvalidFunctionParam(p, k.to_string())
    }

    pub fn expected_var_right(p: Position, k: &str) -> Self {
        Error::ExpectedVarRight(p, k.to_string())
    }

    pub fn unsupported_escape(p: Position, c: char) -> Self {
        Error::UnsupportedEscape(p, c.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match *self {
            // JSON parsing errors
            UnexpectedCharacter {
                ref ch,
                ref line,
                ref column,
            } => write!(
                f,
                "Unexpected character in input: {} at ({}:{})",
                ch, line, column
            ),
            UnexpectedEndOfJson => write!(f, "Unexpected end of JSON input"),
            ExceededDepthLimit => write!(f, "Exceeded depth limit while parsing input"),
            FailedUtf8Parsing => write!(f, "Failed to parse UTF-8 bytes in input"),
            WrongType(ref s) => write!(f, "Wrong type in input, expected: {}", s),

            // Parsing errors
            SyntaxError(ref p, ref t) => write!(f, "{}: Syntax error `{}`", p, t),
            UnterminatedStringLiteral(ref p) => write!(f, "{}: Unterminated string literal", p),
            UnexpectedToken(ref p, ref e, ref a) => {
                write!(f, "{}: Expected `{}`, got `{}`", p, e, a)
            }
            ExpectedTokenBeforeEnd(ref p, ref t) => {
                write!(f, "{}: Expected `{}` before end of expression", p, t)
            }
            InvalidStep(ref p, ref k) => {
                write!(
                    f,
                    "{}: The literal value `{}` cannot be used as a step within a path expression",
                    p, k
                )
            }
            InvalidPredicate(ref p) => write!(
                f,
                "{}: A predicate cannot follow a grouping expression in a step",
                p
            ),
            MultipleGroupBy(ref p) => {
                write!(f, "{}: Each step can only have one grouping expression", p)
            }
            InvalidUnary(ref p, ref k) => {
                write!(
                    f,
                    "{}: The symbol `{}` cannot be used as a unary operator",
                    p, k
                )
            }
            InvalidFunctionParam(ref p, ref k) => {
                write!(f, "{}: Parameter `{}` of function definition must be a variable name (start with $)", p, k)
            }
            ExpectedVarLeft(ref p) => write!(
                f,
                "{}: The left side of `:=` must be a variable name (start with $)",
                p
            ),
            ExpectedVarRight(ref p, ref k) => write!(
                f,
                "{}: The right side of `{}` must be a variable name (start with $)",
                p, k
            ),
            UnterminatedComment(ref p) => write!(f, "{}: Comment has no closing tag", p),
            NumberOutOfRange(ref p, ref n) => write!(f, "{}: Number out of range: {}", p, n),
            InvalidUnicodeEscape(ref p) => write!(
                f,
                "{}: The escape sequence \\u must be followed by 4 hex digits",
                p
            ),
            UnsupportedEscape(ref p, ref c) => {
                write!(f, "{}: Unsupported escape sequence: \\{}", p, c)
            }
            UnterminatedQuoteProp(ref p) => write!(
                f,
                "{}: Quoted property name must be terminated with a backquote (`)",
                p
            ),
        }
    }
}
