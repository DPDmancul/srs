#[macro_export]
macro_rules! test_transpile {
    ($($title: ident : $srs: expr => {$($rs: tt)*})*) => {
        $(
            #[test]
            fn $title() {
                use srs::{parse, rustify};

                use core::str::FromStr;
                use proc_macro2::TokenStream;
                use prettyplease::unparse;
                use pretty_assertions::assert_eq;

                let tok = parse($srs)
                        .into_iter()
                        .map(|e| rustify(&e.unwrap()).unwrap())
                        .collect::<TokenStream>();

                println!("{}", tok);

                let [a, b] = [
                    ("srs", tok),
                    ("rs", TokenStream::from_str(stringify!{$($rs)*}).expect("Cannot tokenize rust version"))
                ].map(|(i, x)| unparse(&syn::parse2(x).expect(&format!("syn cannot parse {}", i))));

                assert_eq!(a, b)
            }
        )*
    };
}
