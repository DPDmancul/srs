mod common;

test_transpile! {
    precedence: r##"
        (fn main ()
          (* (+ 1 2) (* 3 4))
          (+ (+ 1 2) (* 3 4))
          (* (/ 1 2) 3)
          (/ (* 1 2) 3)
          (* 1 (/ 2 3))
          (= a (.. 1 5))
          (as (* (as a u8) (as b u8)) f64))
    "## => {
        fn main() {
            (1 + 2) * (3 * 4);
            1 + 2 + 3 * 4;
            1 / 2 * 3;
            1 * 2 / 3;
            1 * (2 / 3);
            a = 1 .. 5;
            (a as u8 * b as u8) as f64;
        }
    }
}
