macro_rules! token_tree {
    ($variant: ident ($($args: tt)*)) => {
        TokenTree::$variant($variant::new($($args)*))
    };
}

macro_rules! token_stream {
    () => { TokenStream::new() };
    ($($variant: ident ($($args: tt)*)),+) => {
        TokenStream::from_iter([
            $(token_tree!($variant($($args)*))),+
        ])
    };
    ($($args: tt)+) => {
        TokenStream::from(token_tree!($($args)+))
    };
}

pub(crate) use token_tree;
pub(crate) use token_stream;
