use alloc::{string::String, vec::Vec};
use core::{
    fmt::{self, Display},
    mem,
};
use itertools::Itertools;

use crate::Error;

/// Represents an S-expression.
#[derive(Debug)]
pub enum Sexp {
    /// This expression is an atom.
    Atom {
        /// The raw value of the atom.
        val: String,
        /// Its line number in the input.
        lineno: usize,
    },
    /// This expression is a list.
    List(Vec<Sexp>),
    /// This expression is an array.
    Array(Vec<Sexp>),
    /// This expression is a list of generics.
    Generics(Vec<Sexp>),
}

/// An error occurred during parsing.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    /// An unexpected character was found.
    Unexpected(char),
    /// An atom was found outside a list.
    AtomOutsideList(String),
    /// A group closing symbol is missing.
    Missing(char),
    /// There are too much group open/closing symbols.
    TooMuch(char),
    /// A group is closed with the wrong symbol.
    WrongClose(
        /// Opening character.
        char,
        /// Closing character.
        char,
    ),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected(c) => write!(f, "Unexpected '{}'", c),
            Self::AtomOutsideList(a) => write!(f, "Atom '{}' was found outside a list", a),
            Self::Missing(c) => write!(f, "Missing '{}'", c),
            Self::TooMuch(c) => write!(f, "Too much '{}'", c),
            Self::WrongClose(open, close) => write!(f, "'{}' closed by '{}'", open, close),
        }
    }
}

/// Parses a string into an iterator of possible s-expressions.
///
/// If you have an iterator (e.g. stdin or a file) it is preferred to use [`parse_lines`] since the
/// latter doesn't consume the input iterator.
pub fn parse(input: &str) -> impl Iterator<Item = Result<Sexp, Error<ParseError>>> + '_ {
    parse_lines(input.split('\n'))
}

/// Parses an stringifiable (text) iterator into an iterator of possible s-expressions.
///
/// The input iterator is not consumed by this function, so it is preferable to use this, rather
/// than [`parse`], if you have already an iterator for your input (e.g. stdin or a file).
///
/// # Example
///
/// ```
/// use std::io::stdin;
/// use srs::{parse_lines, rustify};
/// # use std::io::BufRead;
/// # let stdin = std::io::Cursor::new("(f x)");
/// # macro_rules! println {
/// #   ("{}", $v: tt) => { assert_eq!("f(x);", $v.to_string()); }
/// # }
/// # macro_rules! eprintln {
/// #   ($($e: tt) *) => { panic!($($e)*); }
/// # }
///
/// for x in parse_lines(stdin.lines().map(Result::unwrap)) {
///     match x {
///         Ok(res) => match srs::rustify(&res) {
///             Ok (res) => println!("{}", res),
///             Err(err) => eprintln!("Error. {}", err),
///         }
///         Err(err) => eprintln!("Parse error. {}", err),
///     }
/// }
/// ```
pub fn parse_lines(
    input: impl Iterator<Item = impl Into<String>>,
) -> impl Iterator<Item = Result<Sexp, Error<ParseError>>> {
    let mut string_mode = false;
    let mut escape_mode = false;

    // Contains expressions not already pushed into resulting iterator
    let mut expressions = Vec::<Sexp>::new();
    // Contains expressions scope stack
    let mut scopes = Vec::<(Vec<Sexp>, char)>::new();

    /// Returns the matching grouping character.
    ///
    /// e.g. `pair_of('(') == ')'`
    fn pair_of(par: char) -> char {
        match par {
            '(' => ')',
            ')' => '(',
            '[' => ']',
            ']' => '[',
            '<' => '>',
            '>' => '<',
            '{' => '}',
            '}' => '{',
            _ => ' ',
        }
    }

    input
        .enumerate()
        .batching(move |iterator| loop {
            match iterator.next() {
                Some((lineno, line)) => {
                    let mut token = String::new();

                    // Lines are 1-based
                    let lineno = lineno + 1;

                    /// Closes the current token, if any.
                    macro_rules! close_token {
                        (reassign) => { token = String::new() };
                        (reassign not_allocate) => {};
                        ($($x: ident)?) => {
                            if !token.is_empty() {
                                if let Some(scope) = scopes.last_mut() {
                                    scope.0.push(Sexp::Atom { val: token, lineno });
                                } else {
                                    return Some(Err(Error{
                                        lineno: Some(lineno),
                                        kind: ParseError::AtomOutsideList(token)
                                    }))
                                }
                                close_token!(reassign $($x)?);
                            }
                        };
                    }

                    let lineno = Some(lineno);

                    for c in line.into().chars() {
                        match c {
                            '\\' => {
                                if string_mode {
                                    token += "\\";
                                    escape_mode = true
                                } else {
                                    return Some(Err(Error {
                                        lineno,
                                        kind: ParseError::Unexpected('\\'),
                                    }));
                                }
                            }
                            '\"' if !escape_mode => {
                                token += &String::from(c);
                                string_mode = !string_mode
                            }
                            _ if string_mode => {
                                token += &String::from(c);
                                escape_mode = false
                            }
                            par @ ('(' | '[' | '<') => {
                                close_token!();
                                scopes.push((Vec::new(), par))
                            }
                            par @ (')' | ']' | '>') => {
                                close_token!();
                                if let Some((closed, opened_by)) = scopes.pop() {
                                    if par != pair_of(opened_by) {
                                        return Some(Err(Error {
                                            lineno,
                                            kind: ParseError::WrongClose(opened_by, par),
                                        }));
                                    }
                                    let closed = match par {
                                        ')' => Sexp::List(closed),
                                        ']' => Sexp::Array(closed),
                                        '>' => Sexp::Generics(closed),
                                        _ => unreachable!(),
                                    };
                                    if let Some(scope) = scopes.last_mut() {
                                        scope.0.push(closed);
                                    } else {
                                        expressions.push(closed)
                                    }
                                } else {
                                    return Some(Err(Error {
                                        lineno,
                                        kind: ParseError::TooMuch(par),
                                    }));
                                }
                            }
                            ';' => {
                                close_token!();
                                break;
                            }
                            ' ' => close_token!(),
                            _ => token += &String::from(c),
                        }
                    }

                    close_token!(not_allocate);

                    if scopes.is_empty() {
                        return Some(Ok(mem::take(&mut expressions)));
                    }
                }
                None if !scopes.is_empty() => {
                    let e = Error {
                        lineno: None,
                        kind: ParseError::Missing(pair_of(scopes.last().unwrap().1)),
                    };
                    scopes.clear(); // Avoids infinite re-entering in this case
                    return Some(Err(e));
                }
                _ => return None,
            }
        })
        // Transpose Result and Vec (required to flatten the inner vecs)
        .flat_map(|r| match r {
            Err(e) => vec![Err(e)],
            Ok(v) => v.into_iter().map(Result::Ok).collect(),
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn errors() {
        use crate::{
            parser::{parse, ParseError},
            Error,
        };
        use alloc::string::String;

        let mut res = parse("a\n\\\n(]\n<>>");
        assert_eq!(
            res.next().unwrap().err().unwrap(),
            Error {
                lineno: Some(1),
                kind: ParseError::AtomOutsideList(String::from("a"))
            }
        );
        assert_eq!(
            res.next().unwrap().err().unwrap(),
            Error {
                lineno: Some(2),
                kind: ParseError::Unexpected('\\')
            }
        );
        assert_eq!(
            res.next().unwrap().err().unwrap(),
            Error {
                lineno: Some(3),
                kind: ParseError::WrongClose('(', ']')
            }
        );
        assert_eq!(
            res.next().unwrap().err().unwrap(),
            Error {
                lineno: Some(4),
                kind: ParseError::TooMuch('>')
            }
        );
    }

    #[test]
    fn missing_closing() {
        use crate::{
            parser::{parse, ParseError},
            Error,
        };

        let mut res = parse("(");
        assert_eq!(
            res.next().unwrap().err().unwrap(),
            Error {
                lineno: None,
                kind: ParseError::Missing(')')
            }
        );
    }
}

