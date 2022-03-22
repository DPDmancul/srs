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

pub(crate) use token_stream;
pub(crate) use token_tree;

macro_rules! interspere_token_stream {
    ($list: expr) => {
        interspere_token_stream!($list, ',')
    };
    ($list: expr, $separator: expr) => {
        interspere_token_stream!($list, $separator, |x| exp_to_token_stream(
            x,
            false,
            i8::MAX
        ))
    };
    ($list: expr, $separator: expr, $mapper: expr) => {{
        #[allow(unused_imports)]
        use ops::PunctAsTokenStream;
        #[allow(unstable_name_collisions)]
        $list
            .into_iter()
            .map($mapper)
            .intersperse_with(|| Ok($separator.punct_as_token_stream()))
            .collect::<Result>()
    }};
}

pub(crate) use interspere_token_stream;
