use std::io::BufRead;

#[derive(Debug)]
pub enum Sexp {
    Atom{val: String, lineno: usize},
    List(Vec<Sexp>),
    Array(Vec<Sexp>),
    Generics(Vec<Sexp>),
}

pub fn parse<T: BufRead>(input: T) -> Vec<Sexp> {
    let mut token = String::new();
    let mut string_mode = false;
    let mut escape_mode = false;

    let mut scopes = vec![(Vec::new(), ' ')];

    for (lineno, line) in input.lines().enumerate().map(|(n, l)| (n, l.unwrap())) {
        let lineno = lineno + 1;

        macro_rules! close_token {
            () => {
                if !token.is_empty() {
                    scopes.last_mut().unwrap().0.push(Sexp::Atom{val: token, lineno});
                    token = String::new();
                }
            };
        }

        for c in line.chars() {
            match c {
                '\\' => {
                    if string_mode {
                        token += "\\";
                        escape_mode = true
                    } else {
                        panic!("Parse error: unexpected '\\' on line {}.", lineno)
                    }
                }
                '\"' if !escape_mode => {
                    token += &String::from(c);
                    string_mode = !string_mode
                }
                _ if string_mode => {
                    token += &String::from(c);
                    escape_mode = false
                }
                par @ ('(' | '[' | '<') => {
                   close_token!();
                   scopes.push((Vec::new(), par))
                }
                par @ (')' | ']' | '>') => {
                    close_token!();
                    let (closed, opened_by) = scopes.pop().unwrap();
                    if !(par == ')' && opened_by == '(' || par == ']' && opened_by == '[' || par == '>' && opened_by == '<') {
                        panic!("'{}' closed by '{}' on line {}", opened_by, par, lineno)
                    }
                    scopes.last_mut().unwrap().0.push(
                        match par {
                            ')' => Sexp::List(closed),
                            ']' => Sexp::Array(closed),
                            '>' => Sexp::Generics(closed),
                            _ => unreachable!()
                        }
                    );
                    if scopes.is_empty() {
                        panic!("Too much '{}' on line {}", par, lineno)
                    }
                }
                ';' => {
                    close_token!();
                    break;
                }
                ' ' => close_token!(),
                _ => token += &String::from(c),
            }
        }
        close_token!()
    }

    if scopes.len() != 1 {
        panic!("Missing ')'.")
    }

    scopes.pop().unwrap().0
}
