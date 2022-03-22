use super::*;

pub fn list_to_token_stream<'a>(
    mut l: impl Iterator<Item = &'a Sexp>,
    statement: bool,
    precedence: i8,
) -> Result {
    // let mut l = l.iter().peekable();

    let mut res = token_stream![];

    while let Some(exp) = l.next() {
        match exp {
            Sexp::Atom { val, lineno } => {
                let lineno = *lineno;
                match val.as_str() {
                    // Operators
                    "!" => {
                        let mut stream = token_stream![Punct('!', Spacing::Alone)];
                        let self_precedence = ops::precedence(val, true);
                        stream.extend(exp_to_token_stream(
                            l.next().ok_or(Error {
                                lineno: Some(lineno),
                                kind: RustifyError::MissingOperand(val.to_string()),
                            })?,
                            false,
                            self_precedence,
                        )?);
                        if l.next().is_some() {
                            return Err(Error {
                                lineno: Some(lineno),
                                kind: RustifyError::TooMuchArguments(val.to_string()),
                            });
                        }
                        res.extend(if self_precedence > precedence {
                            token_stream![Group(Delimiter::Parenthesis, stream)]
                        } else {
                            stream
                        });
                        break;
                    }
                    "." | "::" | "+" | "-" | "*" | "/" | "%" | "|" | "||" | "&mut" | "&" | "&&"
                    | "*mut" | "<<" | ">>" | "@" | "^" | "+=" | "-=" | "*=" | "/=" | "%="
                    | "|=" | "&=" | "<<=" | ">>=" | "^=" | "=" | "==" | "!=" | "<" | "<=" | ">"
                    | ">=" | ".." | "as" => {
                        res.extend(ops::op_to_token_stream(
                            val, l, lineno, statement, precedence,
                        )?);
                        break;
                    }

                    // Public
                    "pub" => res.extend(token_stream![Ident("pub", Span::call_site())]),

                    // Use
                    "use" => {
                        for path in l {
                            res.extend(token_stream![Ident("use", Span::call_site())]);
                            res.extend(path_to_token_stream(path, lineno)?);
                            res.extend(token_stream![Punct(';', Spacing::Alone)]);
                        }
                        break;
                    }

                    // Control flow
                    "if" => todo!(),
                    "match" => {
                        res.extend(flow::match_to_token_stream(l, lineno, statement)?);
                        break;
                    }

                    // Loops
                    "for" => todo!(),
                    "while" => todo!(),
                    "loop" => {
                        res.extend(token_stream![Ident("loop", Span::call_site())]);
                        res.extend(block_to_token_stream(l, !statement)?);
                        break;
                    }

                    // break, continue, return
                    "break" | "continue" | "return" => {
                        res.extend(token_stream!(Ident(val, Span::call_site())));
                        if let Some(a) = l.next() {
                            res.extend(exp_to_token_stream(a, false, i8::MAX))
                        }
                        if l.next().is_some() {
                            return Err(Error {
                                lineno: Some(lineno),
                                kind: RustifyError::TooMuchArguments(val.to_string()),
                            });
                        }
                        if statement {
                            res.extend(token_stream![Punct(';', Spacing::Alone)]);
                        }
                        break;
                    }

                    // Functions & closures (lambdas)
                    "fn" => {
                        res.extend(r#fn::fn_to_token_stream(l, statement, lineno)?);
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
