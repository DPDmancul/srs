#[macro_export]
macro_rules! transpile {
    ($title: ident, $srs: expr, $rs: expr) => {
        #[test]
        fn $title() {
            assert_eq!(
                srs::parse($srs)
                    .into_iter()
                    .map(|e| srs::rustify(&e.unwrap()).unwrap().to_string())
                    .collect::<String>(),
                indoc::indoc!($rs)
            )
        }
    };
}
