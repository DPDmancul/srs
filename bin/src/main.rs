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

    match srs::parser::parse_lines(input.lines().map(Result::unwrap)) {
        Ok(res) => res.iter().for_each(|e| write!(output, "{}", e).unwrap()),
        Err(err) => eprintln!("Parse error. {}", err),
    }
}
