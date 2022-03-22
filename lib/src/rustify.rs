use alloc::string::{String, ToString};
use core::{
    fmt::{self, Display},
    str::FromStr,
};
use itertools::Itertools;
use proc_macro2::*;

use crate::{parser::Sexp, Error};

mod flow;
mod r#fn;
mod list;
mod macros;
mod ops;
use macros::*;

/// An error occurred during generating Rust code.
#[derive(Debug)]
pub enum RustifyError {
    /// Error derived from parsing atoms
    AtomParseError(
        /// Atom representation.
        String,
        /// Occurred error.
        LexError,
    ),
    /// Array or generic instead of function name.
    UnexpectedFunctionName(
        /// The representation of the unexpected token.
        String,
    ),
    /// Arguments are missing for the given constructor.
    MissingArguments(String),
    /// An operand is missing for the given operator.
    MissingOperand(String),
    /// Too much arguments were passed.
    TooMuchArguments(String),
    /// Match condition malformed.
    ExpectedMatchCondition
}

impl Display for RustifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AtomParseError(a, e) => write!(f, "Cannot properly parse atom `{}`. {}", a, e),
            Self::UnexpectedFunctionName(exp) => write!(
                f,
                "Unexpected array or generics `{:?}` as function name",
                exp
            ),
            Self::MissingArguments(x) => write!(f, "Missing arguments for '{}'", x),
            Self::MissingOperand(op) => write!(f, "Missing operands for operator '{}'", op),
            Self::TooMuchArguments(x) => write!(f, "Too much arguments for '{}'", x),
            Self::ExpectedMatchCondition => write!(f, "Expected list (condition value) in match body"),
        }
    }
}

pub type Result = core::result::Result<TokenStream, Error<RustifyError>>;

/// Generates Rust code from a parse s-expression.
/// # Example
///
/// ```
/// use srs::rustify;
/// # let parsed_sexp = srs::parse("(f x)").next().unwrap().unwrap();
/// # macro_rules! println {
/// #   ("{}", $v: tt) => { assert_eq!("f (x) ;", $v.to_string()); }
/// # }
/// # macro_rules! eprintln {
/// #   ($($e: tt) *) => { panic!($($e)*); }
/// # }
///
/// match rustify(&parsed_sexp) {
///     Ok(res) => println!("{}", res),
///     Err(err) => eprintln!("Error. {}", err),
/// }
/// ```
pub fn rustify(exp: &Sexp) -> Result {
    exp_to_token_stream(exp, true)
}

fn exp_to_token_stream(exp: &Sexp, statement: bool) -> Result {
    match exp {
        Sexp::Atom { val, lineno } => match TokenStream::from_str(val) {
            Ok(val) => Ok(val),
            Err(e) => Err(Error {
                lineno: Some(*lineno),
                kind: RustifyError::AtomParseError(val.to_string(), e),
            }),
        },
        Sexp::Array(a) => Ok(token_stream![Group(
            Delimiter::Bracket,
            interspere_token_stream!(a)?,
        )]),
        Sexp::Generics(a) => {
            let mut res = token_stream![Punct('<', Spacing::Joint)];
            res.extend(interspere_token_stream!(a)?);
            res.extend(token_stream![Punct('>', Spacing::Alone)]);
            Ok(res)
        }
        Sexp::List(l) => list::list_to_token_stream(l.iter(), statement),
    }
}

fn call_to_token_stream<'a>(
    name: &Sexp,
    args: impl Iterator<Item = &'a Sexp>,
    statement: bool,
) -> Result {
    let mut res = exp_to_token_stream(name, false)?;
    res.extend(token_stream![Group(
        Delimiter::Parenthesis,
        interspere_token_stream!(args)?,
    )]);
    if statement {
        res.extend(token_stream![Punct(';', Spacing::Alone)])
    }
    Ok(res)
}

/// Writes the body of a block
/// l: expressions
/// returns: returns last expression?
fn block_to_token_stream<'a>(l: impl Iterator<Item = &'a Sexp>, returns: bool) -> Result {
    Ok(token_stream![Group(
        Delimiter::Brace,
        TokenStream::from_iter(
            l.peekable()
                .batching(|it| it
                    .next()
                    .map(|x| exp_to_token_stream(x, !returns || it.peek().is_some())))
                .collect::<Result>()?
        )
    )])
}

fn path_to_token_stream(path: &Sexp, lineno: usize) -> Result {
    match path {
        Sexp::Atom { val, .. } => Ok(TokenStream::from_str(val).map_err(|e| Error {
            lineno: Some(lineno),
            kind: RustifyError::AtomParseError(val.to_string(), e),
        })?),
        Sexp::List(l) => match l.first() {
            Some(Sexp::Atom { val, lineno }) if val == "::" => {
                interspere_token_stream!(l[1..].iter(), "::", |p| path_to_token_stream(p, *lineno))
            }
            _ => Ok(token_stream!(Group(
                Delimiter::Brace,
                interspere_token_stream!(l, ',', |p| path_to_token_stream(p, lineno))?
            ))),
        },
        _ => Err(Error {
            lineno: Some(lineno),
            kind: RustifyError::UnexpectedFunctionName(rustify(path)?.to_string()),
        }),
    }
}
