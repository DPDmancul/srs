use super::*;
use ops::PunctAsTokenStream;

pub fn match_to_token_stream<'a>(
    mut l: impl Iterator<Item = &'a Sexp>,
    lineno: usize,
    statement: bool,
) -> Result {
    let mut res = token_stream![Ident("match", Span::call_site())];
    res.extend(exp_to_token_stream(
        l.next().ok_or(Error {
            lineno: Some(lineno),
            kind: RustifyError::MissingArguments("match".into()),
        })?,
        false,
    )?);

    res.extend(token_stream![Group(
        Delimiter::Brace,
        TokenStream::from_iter(l.map(|m| if let Sexp::List(m) = m {
            let mut res = exp_to_token_stream(m.get(0).ok_or(Error{
                lineno: Some(lineno),
                kind: RustifyError::ExpectedMatchCondition
            })?, false)?;
            res.extend("=>".punct_as_token_stream());
            if m.len() == 2 {
                res.extend(exp_to_token_stream(m.get(1).unwrap(), false));
                res.extend(','.punct_as_token_stream());
            } else {
                res.extend(block_to_token_stream(m[1..].iter(), !statement)?)
            }
            Ok(res)
        } else {
            Err(Error {
                lineno: Some(lineno),
                kind: RustifyError::ExpectedMatchCondition,
            })
        }).collect::<Result>()?)
    )]);

    if statement {
        res.extend(token_stream![Punct(';', Spacing::Alone)]);
    }

    Ok(res)
}
