use alloc::{string::String, vec::Vec};
use core::fmt::{self, Display};

use crate::Error;

#[derive(Debug)]
pub enum Sexp {
    Atom { val: String, lineno: usize },
    List(Vec<Sexp>),
    Array(Vec<Sexp>),
    Generics(Vec<Sexp>),
}

#[derive(Debug)]
pub enum ParseError {
    Unexpected(char),
    Missing(char),
    TooMuch(char),
    WrongClose(char, char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected(c) => write!(f, "Unexpected '{}'", c),
            Self::Missing(c) => write!(f, "Missing '{}'", c),
            Self::TooMuch(c) => write!(f, "Too much '{}'", c),
            Self::WrongClose(open, close) => write!(f, "'{}' closed by '{}'", open, close),
        }
    }
}

pub fn parse(input: &str) -> Result<Vec<Sexp>, Error<ParseError>> {
    parse_lines(input.split('\n'))
}

pub fn parse_lines<T: Iterator<Item = S>, S: Into<String>>(
    input: T,
) -> Result<Vec<Sexp>, Error<ParseError>> {
    let mut token = String::new();
    let mut string_mode = false;
    let mut escape_mode = false;

    let mut scopes = vec![(Vec::new(), ' ')];

    for (lineno, line) in input.enumerate() {
        let lineno = lineno + 1;

        macro_rules! close_token {
            () => {
                if !token.is_empty() {
                    scopes
                        .last_mut()
                        .unwrap()
                        .0
                        .push(Sexp::Atom { val: token, lineno });
                    token = String::new();
                }
            };
        }

        for c in line.into().chars() {
            match c {
                '\\' => {
                    if string_mode {
                        token += "\\";
                        escape_mode = true
                    } else {
                        return Err(Error {
                            lineno: Some(lineno),
                            kind: ParseError::Unexpected('\\'),
                        });
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
                    let (closed, opened_by) = scopes.pop().unwrap();
                    if !(par == ')' && opened_by == '('
                        || par == ']' && opened_by == '['
                        || par == '>' && opened_by == '<')
                    {
                        return Err(Error {
                            lineno: Some(lineno),
                            kind: ParseError::WrongClose(opened_by, par),
                        });
                    }
                    scopes.last_mut().unwrap().0.push(match par {
                        ')' => Sexp::List(closed),
                        ']' => Sexp::Array(closed),
                        '>' => Sexp::Generics(closed),
                        _ => unreachable!(),
                    });
                    if scopes.is_empty() {
                        return Err(Error {
                            lineno: Some(lineno),
                            kind: ParseError::TooMuch(par),
                        });
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
        close_token!()
    }

    if scopes.len() != 1 {
        Err(Error {
            lineno: None,
            kind: ParseError::Missing(')'),
        })
    } else {
        Ok(scopes.pop().unwrap().0)
    }
}
