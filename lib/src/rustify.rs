use alloc::string::{String, ToString};
use core::{
    fmt::{self, Display},
    str::FromStr,
};
use itertools::Itertools;
use proc_macro2::*;

use crate::{parser::Sexp, Error};

mod list;

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

#[macro_export]
macro_rules! token_tree {
    ($variant: ident ( $($args: tt)*)) => {
        TokenTree::$variant($variant::new($($args)*))
    }
}
#[macro_export]
macro_rules! token_stream {
    ($($($args: tt)*,)*) => {
        TokenStream::from_iter([
            $(token_tree!($($args)*))*
        ])
    };
    ($($args: tt)*) => {
        TokenStream::from($crate::token_tree!($($args)*))
    }
}

fn exp_to_token_stream(exp: &Sexp, statement: bool) -> Result {
    // Writes the body of a block
    // l: expressions
    // returns: returns last expression?
    /* let body = |f: &mut fmt::Formatter<'_>,
                mut l: Peekable<core::slice::Iter<Sexp>>,
                level,
                returns: bool| {
        writeln!(f, "{{")?;
        while let Some(e) = l.next() {
            write!(
                f,
                "{}{}",
                indent!(level + 1),
                IndSexp(e, level + 1, !returns || l.peek().is_some())
            )?
        }
        if returns {
            writeln!(f)?
        }
        write!(f, "{}}}", indent!(level))?;
        if statement {
            writeln!(f)?
        }
        Ok(())
    }; */

    match exp {
        Sexp::Atom { val, lineno } => match TokenStream::from_str(val) {
            Ok(val) => Ok(val),
            Err(e) => Err(Error {
                lineno: Some(*lineno),
                kind: RustifyError::AtomParseError(val.to_string(), e),
            }),
        },
        Sexp::Array(a) | Sexp::Generics(a) => {
            Ok(token_stream![Group(
                match exp {
                    Sexp::Array(_) => Delimiter::Bracket,
                    Sexp::Generics(_) => Delimiter::Brace, // TODO angular
                    _ => unreachable!(),
                },
                #[allow(unstable_name_collisions)]
                list_to_token_stream(a, ',')?,
            )])
        }
        Sexp::List(l) => list::list_to_token_stream(l.iter(), statement),
    }
}

fn list_to_token_stream<'a>(list: impl IntoIterator<Item = &'a Sexp>, separator: char) -> Result {
    #[allow(unstable_name_collisions)]
    list.into_iter()
        .map(|x| exp_to_token_stream(x, false))
        .intersperse_with(|| Ok(token_stream![Punct(separator, Spacing::Alone)]))
        .collect::<Result>()
}

fn call_to_token_stream<'a>(
    name: &Sexp,
    args: impl Iterator<Item = &'a Sexp>,
    statement: bool,
) -> Result {
    let mut res = exp_to_token_stream(name, false)?;
    res.extend(token_stream![Group(
        Delimiter::Parenthesis,
        list_to_token_stream(args, ',')?,
    )]);
    if statement {
        res.extend(token_stream![Punct(';', Spacing::Alone)])
    }
    Ok(res)
}

