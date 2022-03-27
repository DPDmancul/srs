mod common;

test_transpile! {
    simple_struct: r##"
        ;; (struct Test
        ;;   :a u8
        ;;   :b String
        ;;   :c bool)

        (fn main ()
          (match x
            ((Test :a 1 :b :c true))
            ((Test :c true :b ..))
            ((Test ..)))
          ((:: Extern Test) :c true (.. f)))
    "## => {
        /* struct Test {
            a: u8,
            b: String,
            c: bool
        } */

        fn main() {
            match x {
                Test {a: 1, b, c: true} => {}
                Test {c: true, b, ..} => {}
                Test {..} => {}
            };
            Extern::Test {c: true, ..f};
        }
    }

     unit_struct: r##"
        (struct Test)

        (fn main ()
          (f Test))
    "## => {
        struct Test;

        fn main() {
            f(Test);
        }
    }

    tuple_struct: r##"
        ;; (struct Test u32 bool)

        (fn main ()
          (Test 1 false))
    "## => {
        // struct Test(u32, bool);

        fn main() {
            Test(1, false);
        }
    }

    /* pub_struct: r##"
        (pub struct Test
          :a u8
          (pub :b String)
          ;; or (pub :b) String
          ;; or :b pub String
          ;; or :b (pub String)
          ;; or other ?
          :c bool)

        (pub struct Tuple u32 (pub bool))
    "## => {
        pub struct Test {
            a: u8,
            pub b: String,
            c: bool
        }

        pub struct Tuple(u32, pub bool);
    } */

    struct_update: r##"
        (fn main ()
         (Test :a 1 :b (.. other)))
    "## => {
        fn main() {
            Test {a: 1, b, ..other};
        }
    }
}
