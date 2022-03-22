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
    fn punct_as_token_stream(&self) -> TokenStream {
        TokenStream::from_str(self).unwrap()
    }
}

pub fn op_to_token_stream<'a>(op: &str, mut operands: impl Iterator<Item = &'a Sexp>, lineno: usize, statement: bool) -> Result {
    let mut res = exp_to_token_stream(operands.next().ok_or(Error{
        lineno: Some(lineno),
        kind: RustifyError::MissingOperand(op.to_string())
    })?, false)?;

    let mut operands = operands.peekable();

    if operands.peek().is_some() {
        // Binary operator
        res.extend(op.punct_as_token_stream());
        res.extend(interspere_token_stream!(operands, op))
    } else {
        // Prefix operator
        let old_res = res;
        res = op.punct_as_token_stream();
        res.extend(old_res)
    }

    if op != "." && op != "::" {
        res = token_stream![Group(Delimiter::Parenthesis, res)];
    }

    if statement {
        res.extend(token_stream![Punct(';', Spacing::Alone)])
    }
    Ok(res)
}
