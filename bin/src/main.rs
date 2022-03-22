use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

/// S-expression to Rust transpiler
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// srs input file. - read from stdin
    #[clap(default_value = "-")]
    input: String,

    /// rs output file. - write to stdout
    #[clap(default_value = "-")]
    output: String,
}

/// Panics with a custom message, without other informations.
macro_rules! clean_panic {
    ($($arg : tt) *) => {{
        std::panic::set_hook(Box::new(|info| {
            if let Some(s) = info.payload().downcast_ref::<String>() {
                eprintln!("{}", s);
            }
        }));
        panic!($($arg)*);
    }};
}

fn main() {
    let args = Args::parse();

    let input: Box<dyn BufRead> = match args.input.as_str() {
        "-" => Box::new(BufReader::new(io::stdin())),
        path => Box::new(BufReader::new(File::open(path).unwrap())),
    };

    let mut output: Box<dyn Write> = match args.output.as_str() {
        "-" => Box::new(BufWriter::new(io::stdout())),
        path => Box::new(BufWriter::new(File::create(path).unwrap())),
    };

    let parsed_exps = srs::parse_lines(input.lines().map(Result::unwrap));
    let token_stream = FromIterator::from_iter(parsed_exps.map(|x| match x {
        Ok(res) => srs::rustify(&res).unwrap_or_else(|e| clean_panic!("Error. {}", e)),
        Err(e) => clean_panic!("Parse error. {}", e),
    }));

    write!(
        output,
        "{}",
        prettyplease::unparse(
            &syn::parse2(token_stream).unwrap_or_else(|e| clean_panic!("Syntax error: {}", e)) // TODO better feedback
        )
    )
    .unwrap()
}
