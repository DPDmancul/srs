#[macro_export]
macro_rules! transpile {
    ($title: ident, $srs: expr, $rs: expr) => {
        #[test]
        fn $title() {
            assert_eq!(
                srs::parser::parse(std::io::Cursor::new($srs))
                    .iter()
                    .map(ToString::to_string)
                    .collect::<String>(),
                indoc::indoc!($rs)
            )
        }
    };
}
