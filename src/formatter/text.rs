use crate::types::{FormatSection, FormatToken, NumberFormat};

/// Format a text value according to the specified number format pattern
///
/// # Arguments
/// * `text` - The text value to format
/// * `format` - The parsed number format pattern
///
/// # Returns
/// * `String` - The formatted text string
pub(super) fn format_text(text: &str, format: &NumberFormat) -> String {
    if let Some(text_section) = &format.text_section {
        format_text_with_section(text, text_section)
    } else {
        // If no text section is defined, return the text as is
        text.to_string()
    }
}

/// Format a text value with a text section
fn format_text_with_section(text: &str, section: &FormatSection) -> String {
    let mut result = String::new();

    for token in &section.tokens {
        match token {
            FormatToken::TextValue => {
                result.push_str(text);
            }
            FormatToken::LiteralChar(c) => {
                result.push(*c);
            }
            FormatToken::QuotedText(quoted_text) => {
                result.push_str(quoted_text);
            }
            _ => {
                // Ignore other tokens in text section
            }
        }
    }

    result
}
