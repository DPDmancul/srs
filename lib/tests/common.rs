#[macro_export]
macro_rules! test_transpile {
    ($($title: ident {$($srs: tt)*} => {$($rs: tt)*})*) => {
        $(
            #[test]
            fn $title() {
                use srs::{parse, rustify};

                use core::str::FromStr;
                use proc_macro2::TokenStream;
                use prettyplease::unparse;

                let [a, b] = [
                    parse(stringify!{$($srs)*})
                        .into_iter()
                        .map(|e| rustify(&e.unwrap()).unwrap())
                        .collect::<TokenStream>(),
                        TokenStream::from_str(stringify!{$($rs)*}).unwrap()
                ].map(|x| unparse(&syn::parse2(x).unwrap()));

                assert_eq!(a, b)
            }
        )*
    };
}
