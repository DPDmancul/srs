mod common;

transpile!(
    hello_world,
    r##"(fn main () (println! "Hello World!"))"##,
    r##"
        fn main() {
            println!("Hello World!");
        }
    "##
);

transpile!(
    guess_game, // TODO
    r##"
        (use (:: rand Rng)
             (:: std ((:: cmp Ordering)
                      io)))

        (fn main ()
         (println! "Guess the number!")

         ;; let secret_number =
         (. ((:: rand thread_rng)) (gen_range (.. 1 101)))
         ;; or
         ((. ((:: rand thread_rng)) gen_range) (.. 1 101))

         (loop
          (println! "Please input your guess.")

          ;;     let mut guess =
          ((:: String new))

          (. ((:: io stdin)) (read_line (&mut guess)) (expect "Failed to read line"))
          ;; or
          ((.((. ((:: io stdin)) read_line) (&mut guess)) expect) "Failed to read line")

          ;;     let guess: u32 =
          (match (. guess (trim) (parse))
           ((Ok num) num)
           ((Err _) (continue)))

          (println! "You guessed: {}" guess)

          (match (. guess (cmp (& secret_number)))
           ((:: Ordering Less) (println! "Too small!"))
           ((:: Ordering Greater) (println! "Too big!"))
           ((:: Ordering Equal) (println! "You win!")
                                (break)))))
    "##,
    r##"
        use rand::Rng;
        use std::{cmp::Ordering, io};
        fn main() {
            println!("Guess the number!");
            rand::thread_rng().gen_range((1 .. 101));
            rand::thread_rng().gen_range((1 .. 101));
            loop {
                println!("Please input your guess.");
                String::new();
                io::stdin().read_line((&mut guess)).expect("Failed to read line");
                io::stdin().read_line((&mut guess)).expect("Failed to read line");
                match guess.trim().parse() {
                    Ok(num) => num,
                    Err(_) => continue,
                }
                println!("You guessed: {}", guess);
                match guess.cmp((& secret_number)) {
                    Ordering::Less => println!("Too small!"),
                    Ordering::Greater => println!("Too big!"),
                    Ordering::Equal => {
                        println!("You win!");
                        break;
                    }
                }
            }
        }
    "##
);
