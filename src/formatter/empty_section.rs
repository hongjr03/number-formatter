//! Empty section handling module
//!
//! This module handles the special case of empty sections in number formats.
//! For example, in the format ";;;" (three semicolons), each section is empty.
//! When a value matches an empty section, it should return an empty string.

/// Check if a section is completely empty (has no tokens at all)
pub fn is_empty_section(tokens_len: usize) -> bool {
    tokens_len == 0
}

/// Format a value with an empty section
/// Returns an empty string
pub fn format_empty_section() -> String {
    String::new()
}
