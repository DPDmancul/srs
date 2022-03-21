#![no_std]

#[macro_use]
extern crate alloc;

use core::fmt::{self, Display};

pub mod parser;
pub use parser::{parse, parse_lines};

pub mod rustify;
pub use rustify::rustify;

/// Represents an error occurred during parsing or rustifying.
///
/// You can print those errors with `eprintln!("{}", error);`
#[derive(Debug, Eq, PartialEq)]
pub struct Error<Kind: Display> {
    /// Line number of the input where the error occurs.
    pub lineno: Option<usize>,
    /// Type of error occurred.
    pub kind: Kind,
}

impl<Kind: Display> Display for Error<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(lineno) = self.lineno {
            write!(f, " on line {}", lineno)?
        }
        write!(f, ".")
    }
}

