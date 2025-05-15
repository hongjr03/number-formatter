use crate::types::{FormatSection, FormatToken, LocaleSettings, NumberFormat};

/// Format a text value according to the specified number format pattern
///
/// # Arguments
/// * `text` - The text value to format
/// * `format` - The parsed number format pattern
/// * `locale` - The locale settings
///
/// # Returns
/// * `String` - The formatted text string
pub(super) fn format_text(text: &str, format: &NumberFormat, locale: &LocaleSettings) -> String {
    if let Some(text_section) = &format.text_section {
        format_text_with_section(text, text_section, locale)
    } else {
        // If no text section is defined, return the text as is
        text.to_string()
    }
}

/// Format a text value with a text section
pub(super) fn format_text_with_section(
    text_to_insert: &str,
    section: &FormatSection,
    locale: &LocaleSettings,
) -> String {
    let mut result = String::new();

    for token in &section.tokens {
        match token {
            FormatToken::TextValue => {
                result.push_str(text_to_insert);
            }
            FormatToken::LiteralChar(c) => {
                result.push(*c);
            }
            FormatToken::QuotedText(quoted_text) => {
                result.push_str(quoted_text);
            }
            FormatToken::CurrencySymbolLocaleDefault => {
                result.push_str(&locale.currency_symbol);
            }
            _ => {
                // Ignore other tokens like Fill, SkipWidth, numeric/date placeholders in text section
            }
        }
    }

    result
}
