use std::fmt::{Display, Formatter, Result};

use crate::parser::Sexp;

struct IndSexp<'a>(&'a Sexp, usize);

impl Display for Sexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}", IndSexp(self, 0))
    }
}

impl<'a> Display for IndSexp<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(exp, level) = self;
        let indent_base = "    ";
        let indentation = indent_base.repeat(*level);
        macro_rules! indent {
            () => {
                indentation
            };
            (+$a: expr) => {
                indent!(level + $a)
            };
            ($a: expr) => {
                indent_base.repeat($a)
            };
        }
        match exp {
            Sexp::Atom { val, .. } => write!(f, "{}", val)?,
            Sexp::Array(a) => {
                write!(f, "[")?;
                for v in a {
                    write!(f, "{}, ", v)?
                }
                write!(f, "]")?
            }
            Sexp::Generics(g) => {
                write!(f, "<")?;
                for v in g {
                    write!(f, "{}, ", v)?
                }
                write!(f, ">")?
            }
            Sexp::List(l) => {
                let mut l = l.iter().peekable();
                let mut is_pub = false;
                macro_rules! is_pub {
                    () => {
                        if is_pub {
                            "pub "
                        } else {
                            ""
                        }
                    };
                }
                while let Some(exp) = l.next() {
                    match exp {
                        Sexp::Atom { val, lineno } => match val.as_str() {
                            // Operators
                            "!" => {
                                write!(
                                    f,
                                    "!({})",
                                    l.next().unwrap_or_else(|| panic!(
                                        "Missing value to negate on line {}.",
                                        lineno
                                    ))
                                )?;
                                if l.next().is_some() {
                                    panic!("Too much values for negation on line {}.", lineno)
                                }
                                break;
                            }
                            op @ ("." | "::") => {
                                let first = l.next().unwrap_or_else(|| {
                                    panic!(
                                        "Missing operands for operator '{}' on line {}.",
                                        op, lineno
                                    )
                                });
                                if l.peek().is_some() {
                                    write!(f, "{}", first)?;
                                    for v in l {
                                        write!(f, "{}{}", op, v)?
                                    }
                                } else {
                                    write!(f, "(|x| x{}{})", op, first)?
                                }
                                break;
                            }
                            op @ ("+" | "-" | "*" | "/" | "%" | "|" | "||" | "&" | "&&" | "<<"
                            | ">>" | "@" | "^" | "+=" | "-=" | "*=" | "/=" | "%=" | "|="
                            | "&=" | "<<=" | ">>=" | "^=" | "=" | "==" | "!=" | "<"
                            | "<=" | ">" | ">=" | ".." | "as") => {
                                let first = l.next().unwrap_or_else(|| {
                                    panic!(
                                        "Missing operands for operator '{}' on line {}.",
                                        op, lineno
                                    )
                                });
                                if l.peek().is_some() {
                                    write!(f, "({}", first)?;
                                    for v in l {
                                        write!(f, " {} {}", op, v)?
                                    }
                                    write!(f, ")")?
                                } else {
                                    write!(f, "({}{})", op, first)?
                                }
                                break;
                            }

                            // Public
                            "pub" => is_pub = true,

                            // Use
                            "use" => {
                                for path in l {
                                    write!(f, "{}use ", indent!())?;
                                    fn parse_path (f:  &mut Formatter<'_>, path: &Sexp) -> Result {
                                        match path {
                                            Sexp::Atom{val,..} => write!(f, "{}", val)?,
                                            Sexp::List(l) => match l.first() {
                                                Some(Sexp::Atom{val, ..}) if val == "::" => {
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
                                            _ => panic!("Unexpected array or generics `{:?}` in use.", path)
                                        }
                                        Ok(())
                                    }
                                    parse_path(f, path)?;
                                    writeln!(f, ";")?;
                                }
                                break;
                            }

                            // Functions & closures (lambdas)
                            "fn" => {
                                if let Sexp::Atom { val, .. } = l.next().unwrap_or_else(|| {
                                    panic!("Missing function arguments on line {}", lineno)
                                }) {
                                    write!(f, "{}{}fn {}", indentation, is_pub!(), val)?;
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
                                    write!(f, "()")?;
                                } else {
                                    // lambda
                                    write!(f, "{}|", indent!())?;
                                    write!(f, "|")?;
                                };
                                writeln!(f, " {{")?;
                                for e in l {
                                    writeln!(f, "{}{};", indent!(+1), e)?
                                }
                                writeln!(f, "{}}}", indent!())?;
                                break;
                            }

                            // Function invocation from atom
                            _ => {
                                write!(f, "{}{}", indent!(), val)?;
                                call(f, l)?;
                                break;
                            }
                        },

                        // Function invocation from list
                        Sexp::List(_) => {
                            write!(f, "{}", IndSexp(exp, *level))?;
                            call(f, l)?;
                            break;
                        }

                        _ => panic!("Unexpected array or generics `{:?}` as function name.", exp),
                    }
                }
            }
        }
        Ok(())
    }
}

fn call<'a, I: IntoIterator<Item = &'a Sexp>>(f: &mut Formatter<'_>, l: I) -> Result {
    write!(f, "(")?;
    let mut first = true;
    for a in l {
        if !first {
            write!(f, ", ")?;
        }
        first = false;
        write!(f, "{}", a)?
    }
    write!(f, ")")
}
