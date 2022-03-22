use super::*;

pub fn list_to_token_stream<'a>(mut l: impl Iterator<Item = &'a Sexp>, statement: bool) -> Result {
    // let mut l = l.iter().peekable();

    let mut res = token_stream![];

    while let Some(exp) = l.next() {
        match &exp {
            Sexp::Atom { val, lineno } => {
                match val.as_str() {
                    // Operators
                    /* "!" => {
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
                    }*/
                    // Public
                    "pub" => res.extend(token_stream![Ident("pub", Span::call_site())]),

                    // Use
                    /* "use" => {
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
                    }*/
                    // Functions & closures (lambdas)
                    "fn" => {
                        res.extend(r#fn::fn_to_token_stream(l, statement, *lineno)?);
                        break;
                    }

                    // Function invocation
                    _ => {
                        res.extend(call_to_token_stream(exp, l, statement)?);
                        break;
                    }
                }
            }
            // Function invocation from list
            Sexp::List(_) => {
                res.extend(call_to_token_stream(exp, l, statement)?);
                break;
            }
            _ => {
                return Err(Error {
                    lineno: None,
                    kind: RustifyError::UnexpectedFunctionName(rustify(exp)?.to_string()),
                })
            }
        }
    }
    Ok(res)
}
