mod common;

test_transpile! {
    empty_enum: r##"
        (enum E)
    "## => {
        enum E {}
    }

     simple_enum: r##"
        (enum Test
          A
          B
          C)

        (fn main ()
          (:: Test A))
    "## => {
        enum Test {
            A,
            B,
            C
        }

        fn main() {
            Test::A;
        }
    }

    c_style_enum: r##"
        (enum Test
          (= A 4)
          B
          C)
    "## => {
        enum Test {
            A = 4,
            B,
            C
        }
    }

    complex_enum: r##"
        (pub enum Test
          Nothing
          (Something u32 u8)
          (LotsOfThings
            :usual_struct_stuff bool
            :blah String))

         (fn main ()
           ((:: Test LotsOfThings) :usual_struct_stuff true :blah ((:: String new))))
    "## => {
        pub enum Test {
            Nothing,
            Something(u32, u8),
            LotsOfThings {
                usual_struct_stuff: bool,
                blah: String,
            }
        }

        fn main() {
            Test::LotsOfThings {usual_struct_stuff: true, blah: String::new()};
        }
    }
}
