use std::{
    fmt::{Display, Formatter, Result},
    iter::Peekable,
};

use crate::parser::Sexp;

struct IndSexp<'a>(
    &'a Sexp,
    /// Indentation level
    usize,
    /// Statement
    bool,
);

impl Display for Sexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}", IndSexp(self, 0, true))
    }
}

impl<'a> Display for IndSexp<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let &Self(exp, level, statement) = self;

        let indent_base = "    ";
        let indentation = indent_base.repeat(level);
        /// Returns indentation string (with a possible level shift)
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

        // Writes a function call
        // l: arguments
        let call = |f: &mut Formatter<'_>, l| {
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
        };

        // Writes the body of a block
        // l: expressions
        // returns: returns last expression?
        let body = |f: &mut Formatter<'_>,
                    mut l: Peekable<std::slice::Iter<Sexp>>,
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
        };

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

                /// Writes pub keyword if necessary
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
                                    fn parse_path(f: &mut Formatter<'_>, path: &Sexp) -> Result {
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
                        },

                        // Function invocation from list
                        Sexp::List(_) => {
                            write!(f, "{}", IndSexp(exp, level, false))?;
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

