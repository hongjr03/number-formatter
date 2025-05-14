//! Number format formatting module
//!
//! This module is responsible for formatting numbers according to parsed number format patterns.
//! The main entry point is the `format_number` function.
//! Number formatting implementation
//!
//! This module implements formatting of numbers according to parsed number format patterns.

use crate::types::{LocaleSettings, NumberFormat};

mod core;
mod exponential;
mod sections;
mod text;

/// Format a number according to the specified number format pattern
///
/// # Arguments
/// * `value` - The numeric value to format
/// * `format` - The parsed number format pattern
/// * `locale` - Locale settings for formatting
///
/// # Returns
/// * `String` - The formatted number string
///
/// # Examples
/// ```
/// use number_format::formatter::format_number;
/// use number_format::parser::parse_number_format;
/// use number_format::types::LocaleSettings;
///
/// let format = parse_number_format("0.00").unwrap();
/// let result = format_number(123.456, &format, &LocaleSettings::default());
/// assert_eq!(result, "123.46");
/// ```
pub fn format_number(value: f64, format: &NumberFormat, locale: &LocaleSettings) -> String {
    // Handle special cases first: text value
    if value.is_nan() && format.text_section.is_some() {
        return text::format_text("NaN", format);
    }

    // Determine which section to use based on value and conditions
    let section = sections::select_section(value, format);

    // Format the number using the selected section
    core::format_value(value, section, locale)
}
