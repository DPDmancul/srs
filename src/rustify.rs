use std::io::Write;

use crate::parser::Sexp;

pub fn write<T: Write, I: IntoIterator<Item = Sexp>>(output: &mut T, tree: I, level: usize) {
    let indentation = "    ".repeat(level);
    let mut tree = tree.into_iter();
    match tree.next() {
        Some(Sexp::List(l)) => write(output, l, level),
        Some(Sexp::Atom { val, lineno }) => match val.as_str() {
            "fn" => {
                if let Sexp::Atom { val, .. } = tree
                    .next()
                    .unwrap_or_else(|| panic!("Missing function arguments on line {}", lineno))
                {
                    write!(output, "{}fn {} (", indentation, val).unwrap();
                    let args = tree.next().unwrap_or_else(|| {
                        panic!("Missing arguments for function {} on line {}", val, lineno)
                    });
                    write!(output, ") ").unwrap();
                } else {
                    // lambda
                    write!(output, "{}|", indentation).unwrap();
                    write!(output, "| ").unwrap();
                };
                writeln!(output, " {{").unwrap();
                write(output, tree.collect::<Vec<_>>(), level + 1);
                write!(output, "{}}}", indentation).unwrap();
            }
            _ => {
                write!(output, "{}{}(", indentation, val).unwrap();
                let mut first = true;
                tree.for_each(|a| {
                    if !first {
                        write!(output, ", ").unwrap();
                    }
                    first = false;
                    match a {
                        Sexp::Atom { val, .. } => write!(output, "{}", val).unwrap(),
                        Sexp::List(l) => write(output, l, 0),
                    }
                });
                writeln!(output, ");").unwrap();
            }
        },
        None => (),
    }
}
