use std::{io::{self, BufReader, BufRead, BufWriter, Write}, fs::File};
use clap::Parser;

mod parser;
mod rustify;

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
        path => Box::new(BufReader::new(File::open(path).unwrap()))
    };

    let mut output: Box<dyn Write> = match args.output.as_str() {
        "-" => Box::new(BufWriter::new(io::stdout())),
        path => Box::new(BufWriter::new(File::create(path).unwrap()))
    };

    rustify::write(&mut output, parser::parse(input), 0);
    writeln!(output);
}
