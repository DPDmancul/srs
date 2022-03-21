use proc_macro2::*;
use core::{
    fmt::{self, Display},
    iter::Peekable,
};

use alloc::string::{String, ToString};

use crate::{parser::Sexp, Error};

/// An error occurred during generating Rust code.
#[derive(Debug, Eq, PartialEq)]
pub enum RustifyError {
    /// Array or generic instead of function name.
    UnexpectedFunctionName(
        /// The representation of the unexpected token.
        String
    ),
}

impl Display for RustifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedFunctionName(exp) => write!(
                f,
                "Unexpected array or generics `{:?}` as function name",
                exp
            ),
        }
    }
}

/// Generates Rust code from a parse s-expression.
/// # Example
///
/// ```
/// use srs::rustify;
/// # let parsed_sexp = srs::parse("(f x)").next().unwrap().unwrap();
/// # macro_rules! println {
/// #   ("{}", $v: tt) => { assert_eq!("f(x);", $v.to_string()); }
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
pub fn rustify(exp: &Sexp) -> Result<TokenStream, Error<RustifyError>> {
    exp_to_token_stream(exp, true)
}

fn exp_to_token_stream(exp: &Sexp, statement: bool) -> Result<TokenStream, Error<RustifyError>> {

    // Writes a function call
    // l: arguments
    /* let call = |f: &mut fmt::Formatter<'_>, l| {
        write!(f, "(")?;
        let mut first = true;
        for a in l {
            if !first {
                write!(f, ", ")?
            }
            first = false;
            write!(f, "{}", IndSexp(a, level, false))?
        }
        write!(f, ")")?;
        if statement {
            writeln!(f, ";")?
        }
        Ok(())
    }; */

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

    /* match &exp {
        Sexp::Atom { val, .. } => {
            scope.raw(val);
        }
        Sexp::Array(a) | Sexp::Generics(a) => {
            scope.raw(match exp {
                Sexp::Array(_) => "[",
                Sexp::Generics(_) => "<",
                _ => unreachable!(),
            });
            for v in a {
                add_to_scope(scope, v)?;
                scope.raw(",");
            }
            scope.raw(match exp {
                Sexp::Array(_) => "]",
                Sexp::Generics(_) => ">",
                _ => unreachable!(),
            });
        }
        Sexp::List(l) => add_list_to_scope(scope, l)?,
    }
    Ok(()) */

    Err(Error{ lineno: None, kind: RustifyError::UnexpectedFunctionName(String::new())})
}

fn list_to_token_stream(l: &[Sexp]) -> Result<(), Error<RustifyError>> {
    let mut l = l.iter().peekable();

    while let Some(exp) = l.next() {
        match exp {
            Sexp::Atom { val, lineno } => {
                /*  match val.as_str() {
                    // Operators
                    "!" => {
                        write!(
                            f,
                            "!({})",
                            IndSexp(
                                l.next().unwrap_or_else(|| panic!(
                                    "Missing value to negate on line {}.",
                                    lineno
                                )),
                                level,
                                false
                            )
                        )?;
                        if l.next().is_some() {
                            panic!("Too much values for negation on line {}.", lineno)
                        }
                        break;
                    }
                    op @ ("." | "::") => {
                        let first = IndSexp(
                            l.next().unwrap_or_else(|| {
                                panic!(
                                    "Missing operands for operator '{}' on line {}.",
                                    op, lineno
                                )
                            }),
                            level,
                            false,
                        );
                        if l.peek().is_some() {
                            write!(f, "{}", first)?;
                            for v in l {
                                write!(f, "{}{}", op, IndSexp(v, level, false))?
                            }
                        } else {
                            write!(f, "(|x| x{}{})", op, first)?
                        }
                        if statement {
                            writeln!(f, ";")?
                        }
                        break;
                    }
                    op
                    @
                    ("+" | "-" | "*" | "/" | "%" | "|" | "||" | "&mut" | "&"
                    | "&&" | "<<" | ">>" | "@" | "^" | "+=" | "-=" | "*=" | "/="
                    | "%=" | "|=" | "&=" | "<<=" | ">>=" | "^=" | "=" | "=="
                    | "!=" | "<" | "<=" | ">" | ">=" | ".." | "as") => {
                        let first = IndSexp(
                            l.next().unwrap_or_else(|| {
                                panic!(
                                    "Missing operands for operator '{}' on line {}.",
                                    op, lineno
                                )
                            }),
                            level,
                            false,
                        );
                        if l.peek().is_some() {
                            write!(f, "({}", first)?;
                            for v in l {
                                write!(f, " {} {}", op, IndSexp(v, level, false))?
                            }
                            write!(f, ")")
                        } else {
                            write!(f, "({} {})", op, first)
                        }?;
                        if statement {
                            writeln!(f, ";")?
                        }
                        break;
                    }

                    // Public
                    "pub" => is_pub = true,

                    // Use
                    "use" => {
                        for path in l {
                            write!(f, "{}use ", indent!())?;
                            fn parse_path(
                                f: &mut fmt::Formatter<'_>,
                                path: &Sexp,
                            ) -> fmt::Result {
                                match path {
                                    Sexp::Atom { val, .. } => write!(f, "{}", val)?,
                                    Sexp::List(l) => match l.first() {
                                        Some(Sexp::Atom { val, .. }) if val == "::" => {
                                            let mut first = true;
                                            for path in &l[1..] {
                                                if !first {
                                                    write!(f, "::")?
                                                }
                                                first = false;
                                                parse_path(f, path)?
                                            }
                                        }
                                        _ => {
                                            let mut first = true;
                                            write!(f, "{{")?;
                                            for path in l {
                                                if !first {
                                                    write!(f, ", ")?
                                                }
                                                first = false;
                                                parse_path(f, path)?
                                            }
                                            write!(f, "}}")?;
                                        }
                                    },
                                    _ => panic!(
                                        "Unexpected array or generics `{:?}` in use.",
                                        path
                                    ),
                                }
                                Ok(())
                            }
                            parse_path(f, path)?;
                            writeln!(f, ";")?
                        }
                        break;
                    }

                    // Block
                    "do" => {
                        body(f, l, level, !statement)?;
                        break;
                    }

                    // Control flow
                    "if" => todo!(),
                    "match" => {
                        writeln!(
                            f,
                            "match {} {{",
                            IndSexp(
                                l.next().unwrap_or_else(|| panic!(
                                    "Missing match argument on line {}.",
                                    lineno
                                )),
                                level,
                                false
                            )
                        )?;
                        for m in l {
                            if let Sexp::List(m) = m {
                                write!(
                                    f,
                                    "{}{} => ",
                                    indent!(+1),
                                    IndSexp(
                                        m.get(0)
                                         .unwrap_or_else(
                                             ||
                                             panic!(
                                                 "Missing expected condition in match body on line {}.",
                                                 lineno)),
                                         level+1,
                                         false
                                         )
                                    )?;
                                if m.len() == 2 {
                                    writeln!(f, "{},", IndSexp(&m[1], level + 1, false))
                                } else {
                                    let mut iter = m.iter().peekable();
                                    iter.next().unwrap();
                                    body(f, iter, level + 1, !statement)
                                }?
                            } else {
                                panic!("Expected list (condition value) in match body on line {}.", lineno)
                            }
                        }
                        write!(f, "{}}}", indent!())?;
                        if statement {
                            writeln!(f)?
                        }
                        break;
                    }

                    // Loops
                    "for" => todo!(),
                    "while" => todo!(),
                    "loop" => {
                        write!(f, "loop ")?;
                        body(f, l, level, !statement)?;
                        break;
                    }

                    // break, continue, return
                    k @ ("break" | "continue" | "return") => {
                        write!(f, "{}", k)?;
                        if let Some(a) = l.next() {
                            write!(f, " {}", a)?
                        }
                        if l.next().is_some() {
                            panic!("Too much arguments for {} on line {}.", k, lineno)
                        }
                        if statement {
                            writeln!(f, ";")?
                        }
                        break;
                    }

                    // Functions & closures (lambdas)
                    "fn" => {
                        if let Sexp::Atom { val, .. } = l.next().unwrap_or_else(|| {
                            panic!("Missing function arguments on line {}", lineno)
                        }) {
                            write!(f, "{}fn {}", is_pub!(), val)?;
                            let args = loop {
                                match l.next().unwrap_or_else(|| {
                                    panic!(
                                        "Missing arguments for function {} on line {}",
                                        val, lineno
                                    )
                                }) {
                                    Sexp::List(a) => break a,
                                    g @ Sexp::Generics(_) => write!(f, "{}", g)?,
                                    _ => panic!(
                                        "Missing arguments for function {} on line {}",
                                        val, lineno
                                    ),
                                }
                            };
                            write!(f, "() ")
                        } else {
                            // lambda
                            write!(f, "{}|", indent!())?;
                            write!(f, "| ")
                        }?;
                        body(f, l, level, false)?;
                        break;
                    }

                    // Function invocation from atom
                    _ => {
                        write!(f, "{}", val)?;
                        call(f, l)?;
                        break;
                    }
                }*/
            }

            // Function invocation from list
            Sexp::List(_) => {
                /* write!(f, "{}", IndSexp(exp, level, false))?;
                call(f, l)?;
                break; */
            }

            _ => {
                return Err(Error {
                    lineno: None,
                    kind: RustifyError::UnexpectedFunctionName(rustify(exp)?.to_string()),
                })
            }
        }
    }
    Ok(())
}

