use super::*;

pub trait PunctAsTokenStream {
    fn punct_as_token_stream(&self) -> TokenStream;
}

impl PunctAsTokenStream for char {
    fn punct_as_token_stream(&self) -> TokenStream {
        token_stream![Punct(*self, Spacing::Alone)]
    }
}

impl PunctAsTokenStream for &str {
    #[inline]
    fn punct_as_token_stream(&self) -> TokenStream {
        TokenStream::from_str(self).unwrap()
    }
}

/// Low values means higher precedence.
pub fn precedence(op: &str, unary: bool) -> i8 {
    1 + 2 * match op {
        "." | "::" => 0,
        // Method calls => 1,
        // Field expressions => 2,
        // Function calls, array indexing => 3,
        "?" => 4,
        "-" | "!" | "&" | "&mut" | "*" | "*mut" if unary => 5,
        "as" => 6,
        "*" | "/" | "%" => 7,
        "+" | "-" => 8,
        "<<" | ">>" => 9,
        "&" => 10,
        "^" => 11,
        "|" => 12,
        "==" | "!=" | "<" | ">" | "<=" | ">=" => 13,
        "&&" => 14,
        "||" => 15,
        ".." | "..=" => 16,
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" => 17,
        _ => i8::MAX / 2,
    }
}

#[test]
fn max_precedence() {
    assert_eq!(precedence("", false), i8::MAX)
}

fn associativity(op: &str, unary: bool) -> i8 {
    if unary {
        0
    } else {
        match op {
            "as" | "*" | "/" | "%" | "+" | "-" | "<<" | ">>" | "&" | "^" | "|" | "&&" | "||" => -1, // Left to right
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" => 1, // Right to left
            _ => 0,
        }
    }
}

pub fn op_to_token_stream<'a>(
    op: &str,
    operands: impl Iterator<Item = &'a Sexp>,
    lineno: usize,
    statement: bool,
    parent_precedence: i8,
) -> Result {
    let mut operands = operands.multipeek();

    if operands.peek().is_none() {
        return Err(Error {
            lineno: Some(lineno),
            kind: RustifyError::MissingOperand(op.to_string()),
        });
    }

    let unary = operands.peek().is_none();
    let precedence = precedence(op, unary);

    let mut res = if unary {
        // Prefix operator
        let mut res = op.punct_as_token_stream();
        res.extend(exp_to_token_stream(
            operands.next().unwrap(),
            false,
            precedence,
        )?);
        res
    } else {
        // Binary operator
        let mut precedence = precedence;
        let mut assoc = associativity(op, unary);
        interspere_token_stream!(operands, op, |x| exp_to_token_stream(x, false, {
            let p = precedence;
            if assoc != 0 {
                precedence += assoc;
                assoc = 0;
            }
            p
        }))?
    };

    if precedence > parent_precedence {
        res = token_stream![Group(Delimiter::Parenthesis, res)];
    }

    if statement {
        res.extend(token_stream![Punct(';', Spacing::Alone)])
    }
    Ok(res)
}
