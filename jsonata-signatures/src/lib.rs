use bitflags::bitflags;
use std::cmp;

use jsonata_errors::{Error, Result};

bitflags! {
    pub struct Flags: u8 {
        const ONE_OR_MORE = 0b00000001;
        const OPTIONAL = 0b00000010;
        const ACCEPT_CONTEXT = 0b00000100;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArgKind {
    Null,
    Bool,
    Number,
    String,
    Object,
    Array(Option<Box<ArgKind>>),
    Function(Option<Vec<Arg>>),
    Or(Vec<ArgKind>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    pub kind: ArgKind,
    pub flags: Flags,
}

impl Arg {
    fn new(kind: ArgKind) -> Self {
        Self {
            kind,
            flags: Flags::empty(),
        }
    }
}

pub fn parse(sig: &str) -> Result<Vec<Arg>> {
    let mut index = 0;

    // Empty signatures are not valid
    if sig.len() < 3 {
        return Err(Error::F0401UnexpectedEndOfSignature);
    }

    // All signatures must start with <...
    if &sig[index..index + 1] != "<" {
        return Err(Error::F0402SignatureStartInvalid);
    }
    index += 1;

    let args = parse_signature(sig, &mut index)?;

    if index > sig.len() - 1 {
        return Err(Error::F0401UnexpectedEndOfSignature);
    }

    // ...and end with '>'
    if &sig[index..index + 1] != ">" {
        return Err(Error::F0403SignatureEndInvalid);
    }
    index += 1;

    if index < sig.len() {
        return Err(Error::F0404UnexpectedCharsAtEndOfSignature);
    }

    Ok(args)
}

// Type specifiers:
// b - boolean
// n - number
// s - string
// l - null
// a - array
// o - object
// f - function
// u - (bnsl)
// j - (bnsloa)
// x - (bnsloaf)

// Flags:
// + - one or more
// ? - optional
// - - use context if missing

fn parse_signature(sig: &str, index: &mut usize) -> Result<Vec<Arg>> {
    let mut args = Vec::with_capacity(sig.len());

    loop {
        if *index > sig.len() - 2 {
            return Ok(args);
        }

        match &sig[*index..*index + 1] {
            "?" => {
                *index += 1;
                match args.last_mut() {
                    Some(arg) => arg.flags.insert(Flags::OPTIONAL),
                    None => {
                        return Err(Error::F0405OptionalShouldComeAfterType);
                    }
                }
            }
            "-" => {
                *index += 1;
                match args.last_mut() {
                    Some(arg) => arg.flags.insert(Flags::ACCEPT_CONTEXT),
                    None => {
                        return Err(Error::F0406AllowContextShouldComeAfterType);
                    }
                }
            }
            "+" => {
                *index += 1;
                match args.last_mut() {
                    Some(arg) => arg.flags.insert(Flags::ONE_OR_MORE),
                    None => {
                        return Err(Error::F0407OneOrMoreShouldComeAfterType);
                    }
                }
            }
            "b" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Bool));
            }
            "n" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Number));
            }
            "s" => {
                *index += 1;
                args.push(Arg::new(ArgKind::String));
            }
            "l" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Null));
            }
            "o" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Object));
            }
            ">" => {
                return Ok(args);
            }
            "a" => {
                *index += 1;
                if *index < sig.len() - 2 && &sig[*index..*index + 1] == "<" {
                    *index += 1;
                    let inner = parse_signature(sig, index)?;
                    match inner.len().cmp(&1) {
                        cmp::Ordering::Less => {
                            return Err(Error::F0408NoTypeBetweenCarets);
                        }
                        cmp::Ordering::Equal => args.push(Arg::new(ArgKind::Array(Some(
                            Box::new(inner[0].kind.clone()),
                        )))),
                        cmp::Ordering::Greater => {
                            return Err(Error::F0409MultipleTypesInArray);
                        }
                    };
                    if *index > sig.len() - 2 {
                        return Err(Error::F0410UnterminatedCaret);
                    }
                    if &sig[*index..*index + 1] != ">" {
                        return Err(Error::F0413ExpectedInSignature(">".to_string()));
                    }
                    *index += 1;
                    if *index > sig.len() - 2 {
                        return Ok(args);
                    }
                } else {
                    args.push(Arg::new(ArgKind::Array(None)));
                }
            }
            "(" => {
                // TODO: "S0402": "Choice groups containing parameterized types are not supported",

                *index += 1;
                let mut inner = parse_signature(sig, index)?;
                if inner.is_empty() {
                    return Err(Error::F0411NoTypeBetweenParens);
                }
                if *index > sig.len() - 2 {
                    return Err(Error::F0412UnterminatedParen);
                }
                if &sig[*index..*index + 1] != ")" {
                    return Err(Error::F0413ExpectedInSignature(")".to_string()));
                }
                *index += 1;
                let kinds: Vec<ArgKind> = inner.drain(..).map(|a| a.kind).collect();
                args.push(Arg::new(ArgKind::Or(kinds)));
                if *index > sig.len() - 2 {
                    return Ok(args);
                }
            }
            ")" => {
                return Ok(args);
            }
            "f" => {
                *index += 1;
                if *index < sig.len() - 2 && &sig[*index..*index + 1] == "<" {
                    *index += 1;
                    let inner = parse_signature(sig, index)?;
                    if inner.is_empty() {
                        panic!("No type specified between '<' and '>'")
                    } else {
                        args.push(Arg::new(ArgKind::Function(Some(inner))));
                    }
                    if *index > sig.len() - 2 {
                        panic!("Unterminated '>' in signature")
                    }
                    if &sig[*index..*index + 1] != ">" {
                        panic!("Expected '>' in signature")
                    }
                    *index += 1;
                    if *index > sig.len() - 2 {
                        return Ok(args);
                    }
                } else {
                    args.push(Arg::new(ArgKind::Function(None)));
                }
            }
            "u" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null,
                ])))
            }
            "j" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null,
                    ArgKind::Object,
                    ArgKind::Array(None),
                ])))
            }
            "x" => {
                *index += 1;
                args.push(Arg::new(ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null,
                    ArgKind::Object,
                    ArgKind::Array(None),
                    ArgKind::Function(None),
                ])))
            }
            ":" => {
                // Return types do nothing, skip to the end
                while *index < sig.len() - 1 {
                    *index += 1
                }
                return Ok(args);
            }
            c => {
                return Err(Error::F0414UnexpectedCharInSignature(c.to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool() {
        let sig = parse("<b>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Bool,
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn string() {
        let sig = parse("<s>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::String,
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn number() {
        let sig = parse("<n>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Number,
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn null() {
        let sig = parse("<l>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Null,
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn array() {
        let sig = parse("<a>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Array(None),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn array_with_type() {
        let sig = parse("<a<s>>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Array(Some(Box::new(ArgKind::String))),
                flags: Flags::empty()
            }
        );
    }

    #[test]
    fn nested_arrays_with_type() {
        let sig = parse("<a<a<s>>>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Array(Some(Box::new(ArgKind::Array(Some(Box::new(
                    ArgKind::String
                )))))),
                flags: Flags::empty()
            }
        );
    }

    #[test]
    fn multiple() {
        let sig = parse("<bsl>").unwrap();
        assert_eq!(sig.len(), 3);
        assert_eq!(
            sig,
            [
                Arg {
                    kind: ArgKind::Bool,
                    flags: Flags::empty()
                },
                Arg {
                    kind: ArgKind::String,
                    flags: Flags::empty()
                },
                Arg {
                    kind: ArgKind::Null,
                    flags: Flags::empty()
                },
            ]
        )
    }

    #[test]
    fn optional() {
        let sig = parse("<b?>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Bool,
                flags: Flags::OPTIONAL
            }
        )
    }

    #[test]
    fn accepts_context() {
        let sig = parse("<b->").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Bool,
                flags: Flags::ACCEPT_CONTEXT
            }
        )
    }

    #[test]
    fn one_or_more() {
        let sig = parse("<b+>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Bool,
                flags: Flags::ONE_OR_MORE
            }
        )
    }

    #[test]
    fn or() {
        let sig = parse("<(bsn)>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Or(vec![ArgKind::Bool, ArgKind::String, ArgKind::Number]),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn u() {
        let sig = parse("<u>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null
                ]),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn j() {
        let sig = parse("<j>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null,
                    ArgKind::Object,
                    ArgKind::Array(None)
                ]),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn x() {
        let sig = parse("<x>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Or(vec![
                    ArgKind::Bool,
                    ArgKind::Number,
                    ArgKind::String,
                    ArgKind::Null,
                    ArgKind::Object,
                    ArgKind::Array(None),
                    ArgKind::Function(None)
                ]),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn function() {
        let sig = parse("<f>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Function(None),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn function_with_signature() {
        let sig = parse("<f<abs>>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Function(Some(vec![
                    Arg {
                        kind: ArgKind::Array(None),
                        flags: Flags::empty()
                    },
                    Arg {
                        kind: ArgKind::Bool,
                        flags: Flags::empty()
                    },
                    Arg {
                        kind: ArgKind::String,
                        flags: Flags::empty()
                    }
                ])),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn function_with_complex_signature() {
        let sig = parse("<f<a<f<n>>>>").unwrap();
        assert_eq!(sig.len(), 1);
        assert_eq!(
            sig[0],
            Arg {
                kind: ArgKind::Function(Some(vec![Arg {
                    kind: ArgKind::Array(Some(Box::new(ArgKind::Function(Some(vec![Arg {
                        kind: ArgKind::Number,
                        flags: Flags::empty()
                    }]))))),
                    flags: Flags::empty()
                },])),
                flags: Flags::empty()
            }
        )
    }

    #[test]
    fn replace_sig() {
        let sig = parse("<s-(sf)(sf)n?:s>").unwrap();
        assert_eq!(sig.len(), 4);
        assert_eq!(
            sig,
            vec![
                Arg {
                    kind: ArgKind::String,
                    flags: Flags::ACCEPT_CONTEXT
                },
                Arg {
                    kind: ArgKind::Or(vec![ArgKind::String, ArgKind::Function(None)]),
                    flags: Flags::empty()
                },
                Arg {
                    kind: ArgKind::Or(vec![ArgKind::String, ArgKind::Function(None)]),
                    flags: Flags::empty()
                },
                Arg {
                    kind: ArgKind::Number,
                    flags: Flags::OPTIONAL
                }
            ]
        )
    }
}
