use std::{char, error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    // JSON parsing errors
    I0201UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
    I0202UnexpectedEndOfJson,
    I0203ExceededDepthLimit,
    I0204FailedUtf8Parsing,
    I0205WrongType(String),

    // Compile time errors
    S0101UnterminatedStringLiteral(usize),
    S0102LexedNumberOutOfRange(usize, String),
    S0103UnsupportedEscape(usize, char),
    S0104InvalidUnicodeEscape(usize),
    S0105UnterminatedQuoteProp(usize),
    S0106UnterminatedComment(usize),
    S0201SyntaxError(usize, String),
    S0202UnexpectedToken(usize, String, String),
    S0204UnknownOperator(usize, String),
    S0203ExpectedTokenBeforeEnd(usize, String),
    S0208InvalidFunctionParam(usize, String),
    S0209InvalidPredicate(usize),
    S0210MultipleGroupBy(usize),
    S0211InvalidUnary(usize, String),
    S0212ExpectedVarLeft(usize),
    S0213InvalidStep(usize, String),
    S0214ExpectedVarRight(usize, String),

    // Runtime errors
    D1001NumberOfOutRange(f64),
    D1002NegatingNonNumeric(usize, String),
    D1009MultipleKeys(usize, String),

    // Type errors
    T0410ArgumentNotValid(usize, usize, String),
    T0412ArgumentMustBeArrayOfType(usize, usize, String, String),
    T1003NonStringKey(usize, String),
    T1005InvokedNonFunctionSuggest(usize, String),
    T1006InvokedNonFunction(usize),
    T2001LeftSideNotNumber(usize, String),
    T2002RightSideNotNumber(usize, String),
    T2003LeftSideNotInteger(usize),
    T2004RightSideNotInteger(usize),
    T2009BinaryOpMismatch(usize, String, String, String),
    T2010BinaryOpTypes(usize, String),
}

impl error::Error for Error {}

impl Error {
    /**
     * Error codes
     *
     * Ixxxx    - JSON parsing errors
     * Sxxxx    - Static errors (compile time)
     * Txxxx    - Type errors
     * Dxxxx    - Dynamic errors (evaluate time)
     *  01xx    - tokenizer
     *  02xx    - parser
     *  03xx    - regex parser
     *  04xx    - function signature parser/evaluator
     *  10xx    - evaluator
     *  20xx    - operators
     *  3xxx    - functions (blocks of 10 for each function)
     */
    pub fn code(&self) -> &str {
        match *self {
            // JSON parsing errors
            Error::I0201UnexpectedCharacter { .. } => "I0201",
            Error::I0202UnexpectedEndOfJson => "I0202",
            Error::I0203ExceededDepthLimit => "I0203",
            Error::I0204FailedUtf8Parsing => "I0204",
            Error::I0205WrongType(..) => "I0205",

            // Compile time errors
            Error::S0101UnterminatedStringLiteral(..) => "S0101",
            Error::S0102LexedNumberOutOfRange(..) => "S0102",
            Error::S0103UnsupportedEscape(..) => "S0103",
            Error::S0104InvalidUnicodeEscape(..) => "S0104",
            Error::S0105UnterminatedQuoteProp(..) => "S0105",
            Error::S0106UnterminatedComment(..) => "S0106",
            Error::S0201SyntaxError(..) => "S0201",
            Error::S0202UnexpectedToken(..) => "S0202",
            Error::S0203ExpectedTokenBeforeEnd(..) => "S0203",
            Error::S0204UnknownOperator(..) => "S0204",
            Error::S0208InvalidFunctionParam(..) => "S0208",
            Error::S0209InvalidPredicate(..) => "S0209",
            Error::S0210MultipleGroupBy(..) => "S0210",
            Error::S0211InvalidUnary(..) => "S0211",
            Error::S0212ExpectedVarLeft(..) => "S0212",
            Error::S0213InvalidStep(..) => "S0213",
            Error::S0214ExpectedVarRight(..) => "S0214",

            // Runtime errors
            Error::D1001NumberOfOutRange(..) => "D1001",
            Error::D1002NegatingNonNumeric(..) => "D1002",
            Error::D1009MultipleKeys(..) => "D1009",

            // Type errors
            Error::T0410ArgumentNotValid(..) => "T0410",
            Error::T0412ArgumentMustBeArrayOfType(..) => "T0412",
            Error::T1003NonStringKey(..) => "T1003",
            Error::T1005InvokedNonFunctionSuggest(..) => "T1005",
            Error::T1006InvokedNonFunction(..) => "T1006",
            Error::T2001LeftSideNotNumber(..) => "T2001",
            Error::T2002RightSideNotNumber(..) => "T2002",
            Error::T2003LeftSideNotInteger(..) => "T2003",
            Error::T2004RightSideNotInteger(..) => "T2004",
            Error::T2009BinaryOpMismatch(..) => "T2009",
            Error::T2010BinaryOpTypes(..) => "T2010",
        }
    }
}  

impl fmt::Display for Error {
    #[allow(clippy::many_single_char_names)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        write!(f, "{} @ ", self.code())?;

        match *self {
            // JSON parsing errors
            I0201UnexpectedCharacter { ref ch, ref line, ref column, } =>
                write!(f, "Unexpected character in input: {} at ({}:{})", ch, line, column),
            I0202UnexpectedEndOfJson =>
                write!(f, "Unexpected end of JSON input"),
            I0203ExceededDepthLimit =>
                write!(f, "Exceeded depth limit while parsing input"),
            I0204FailedUtf8Parsing =>
                write!(f, "Failed to parse UTF-8 bytes in input"),
            I0205WrongType(ref s) =>
                write!(f, "Wrong type in input, expected: {}", s),
                
            // Compile time errors
            S0101UnterminatedStringLiteral(ref p) =>
                write!(f, "{}: String literal must be terminated by a matching quote", p),
            S0102LexedNumberOutOfRange(ref p, ref n) =>
                write!(f, "{}: Number out of range: {}", p, n),
            S0103UnsupportedEscape(ref p, ref c) =>
                write!(f, "{}: Unsupported escape sequence: \\{}", p, c),
            S0104InvalidUnicodeEscape(ref p) =>
                write!(f, "{}: The escape sequence \\u must be followed by 4 hex digits", p),
            S0105UnterminatedQuoteProp(ref p) =>
                write!(f, "{}: Quoted property name must be terminated with a backquote (`)", p),
            S0106UnterminatedComment(ref p) =>
                write!(f, "{}: Comment has no closing tag", p),
            S0201SyntaxError(ref p, ref t) =>
                write!(f, "{}: Syntax error `{}`", p, t),
            S0202UnexpectedToken(ref p, ref e, ref a) =>
                write!(f, "{}: Expected `{}`, got `{}`", p, e, a),
            S0203ExpectedTokenBeforeEnd(ref p, ref t) =>
                write!(f, "{}: Expected `{}` before end of expression", p, t),
            S0204UnknownOperator(ref p, ref t) =>
                write!(f, "{}: Unknown operator: `{}`", p, t),
            S0208InvalidFunctionParam(ref p, ref k) =>
                write!(f, "{}: Parameter `{}` of function definition must be a variable name (start with $)", p, k),
            S0209InvalidPredicate(ref p) =>
                write!(f, "{}: A predicate cannot follow a grouping expression in a step", p),
            S0210MultipleGroupBy(ref p) =>
                write!(f, "{}: Each step can only have one grouping expression", p),
            S0211InvalidUnary(ref p, ref k) =>
                write!(f, "{}: The symbol `{}` cannot be used as a unary operator", p, k),
            S0212ExpectedVarLeft(ref p) =>
                write!(f, "{}: The left side of `:=` must be a variable name (start with $)", p),
            S0213InvalidStep(ref p, ref k) =>
                write!(f, "{}: The literal value `{}` cannot be used as a step within a path expression", p, k),
            S0214ExpectedVarRight(ref p, ref k) =>
                write!(f, "{}: The right side of `{}` must be a variable name (start with $)", p, k),
            
            // Runtime errors
            D1001NumberOfOutRange(ref n) =>
                write!(f, "Number out of range: {}", n),
            D1002NegatingNonNumeric(ref p, ref v) =>
                write!(f, "{}: Cannot negate a non-numeric value `{}`", p, v),
            D1009MultipleKeys(ref p, ref k) =>
                write!( f, "{}: Multiple key definitions evaluate to same key: {}", p, k),
            
            // Type errors
            T0410ArgumentNotValid(ref p, ref i, ref t) =>
                write!(f, "{}: Argument {} of function {} does not match function signature", p, i, t),
            T0412ArgumentMustBeArrayOfType(ref p, ref i, ref t, ref ty) =>
                write!(f, "{}: Argument {} of function {} must be an array of {}", p, i, t, ty),
            T1003NonStringKey(ref p, ref v) =>
                write!( f, "{}: Key in object structure must evaluate to a string; got: {}", p, v),
            T1005InvokedNonFunctionSuggest(ref p, ref t) =>
                write!(f, "{}: Attempted to invoke a non-function. Did you mean ${}?", p, t),
            T1006InvokedNonFunction(ref p) =>
                write!(f, "{}: Attempted to invoke a non-function", p),
            T2001LeftSideNotNumber(ref p, ref o) =>
                write!( f, "{}: The left side of the `{}` operator must evaluate to a number", p, o),
            T2002RightSideNotNumber(ref p, ref o) =>
                write!( f, "{}: The right side of the `{}` operator must evaluate to a number", p, o),
            T2003LeftSideNotInteger(ref p) =>
                write!(f, "{}: The left side of the range operator (..) must evaluate to an integer", p),
            T2004RightSideNotInteger(ref p) =>
                write!(f, "{}: The right side of the range operator (..) must evaluate to an integer", p),
            T2009BinaryOpMismatch(ref p,ref l ,ref r ,ref o ) =>
                write!(f, "{}: The values {} and {} either side of operator {} must be of the same data type", p, l, r, o),
            T2010BinaryOpTypes(ref p, ref o) =>
                write!(f, "{}: The expressions either side of operator `{}` must evaluate to numeric or string values", p, o),
        }
    }
}

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
// "T0411": "Context value is not a compatible type with argument {{index}} of function {{token}}",
// "D1004": "Regular expression matches zero length string",
// "T1007": "Attempted to partially apply a non-function. Did you mean ${{{token}}}?",
// "T1008": "Attempted to partially apply a non-function",
// // "T1010": "The matcher function argument passed to function {{token}} does not return the correct object structure",
// "D2005": "The left side of := must be a variable name (start with $)",  // defunct - replaced by S0212 parser error
// "T2006": "The right side of the function application operator ~> must be a function",
// "T2007": "Type mismatch when comparing values {{value}} and {{value2}} in order-by clause",
// "T2008": "The expressions within an order-by clause must evaluate to numeric or string values",
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