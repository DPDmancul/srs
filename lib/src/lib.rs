#![no_std]

#[macro_use]
extern crate alloc;

use core::fmt::{self, Display};

pub mod parser;
pub mod rustify;

#[derive(Debug)]
pub struct Error<Kind: Display> {
    pub lineno: Option<usize>,
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

