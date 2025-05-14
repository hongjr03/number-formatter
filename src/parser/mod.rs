//! Number format parsing module
//!
//! This module is responsible for parsing number format strings and converting them into internal TokenTree representation.
//! The main entry point is the `parse_number_format` function.

mod tokens;
mod combinators;
mod sections;
mod format;

pub use format::parse_number_format; 