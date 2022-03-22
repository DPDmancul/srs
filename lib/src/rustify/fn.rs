use super::*;

pub fn fn_to_token_stream<'a>(
    mut l: impl Iterator<Item = &'a Sexp>,
    statement: bool,
    lineno: usize,
) -> Result {
    let mut res = token_stream![Ident("fn", Span::call_site())];
    if let Sexp::Atom { val, lineno } = l
        .next()
        .unwrap_or_else(|| panic!("Missing function arguments on line {}", lineno))
    {
        res.extend(token_stream![
            Ident(val, Span::call_site()),
            Group(Delimiter::Parenthesis, token_stream![])
        ]);
        /* let args = loop {
            match l.next().unwrap_or_else(|| {
                panic!("Missing arguments for function {} on line {}", val, lineno)
            }) {
                Sexp::List(a) => break a,
                g @ Sexp::Generics(_) => write!(f, "{}", g)?,
                _ => panic!("Missing arguments for function {} on line {}", val, lineno),
            }
        }; */
    } else {
        todo!();
        // lambda
        /* write!(f, "{}|", indent!())?;
        write!(f, "| ") */
    };
    res.extend(block_to_token_stream(l, false)?);
    Ok(res)
}
