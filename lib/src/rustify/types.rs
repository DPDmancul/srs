use super::*;

pub fn struct_to_token_stream<'a>(mut l: impl Iterator<Item = &'a Sexp>, lineno: usize) -> Result {
    let mut res = token_stream![Ident("struct", Span::call_site())];
    match l.next() {
        Some(Sexp::Atom { val, .. }) => {
            res.extend(token_stream![
                Ident(val, Span::call_site()),
                Punct(';', Spacing::Alone)
            ]);
            Ok(res)
        }
        _ => Err(Error {
            lineno: Some(lineno),
            kind: RustifyError::MissingArguments("struct definition".into()),
        }),
    }
}

pub fn enum_to_token_stream<'a>(mut l: impl Iterator<Item = &'a Sexp>, lineno: usize) -> Result {
    let mut res = token_stream![Ident("enum", Span::call_site())];
    match l.next() {
        Some(Sexp::Atom { val, .. }) => {
            res.extend(token_stream![
                Ident(val, Span::call_site()),
                Group(Delimiter::Brace, interspere_token_stream!(l)?)
            ]);
            Ok(res)
        }
        _ => Err(Error {
            lineno: Some(lineno),
            kind: RustifyError::MissingArguments("enum definition".into()),
        }),
    }
}
