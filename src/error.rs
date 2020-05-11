// Error codes
//  Sxxxx    - Static errors (compile time)
//  Txxxx    - Type errors
//  Dxxxx    - Dynamic errors (evaluate time)
//   01xx    - tokenizer
//   02xx    - parser
//   03xx    - regex parser
//   04xx    - function signature parser/evaluator
//   10xx    - evaluator
//   20xx    - operators
//   3xxx    - functions (blocks of 10 for each function)

#[derive(Debug)]
pub struct Error {
    pub code: &'static str,
    pub position: usize,
    pub message: String,
}
