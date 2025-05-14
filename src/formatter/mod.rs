//! Number format formatting module
//!
//! This module is responsible for formatting numbers according to parsed number format patterns.
//! The main entry point is the `format_number` function.
//! Number formatting implementation
//!
//! This module implements formatting of numbers according to parsed number format patterns.

use crate::types::{FormatSection, LocaleSettings, NumberFormat};

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

    let section_to_use = sections::select_section(value, format);

    let original_value_for_sign_check = value;
    let mut value_for_formatting_placeholders = value; // Default to original value
    let mut is_fallback_for_negative = false;

    if value < 0.0 {
        // Determine if the positive_section is being used as a fallback for a negative value.
        // This occurs if:
        // 1. The original value is negative.
        // 2. No specific negative_section is defined in the NumberFormat.
        // 3. The section actually selected by `sections::select_section` IS the `positive_section`.
        if format.negative_section.is_none() {
            // Compare pointers to ensure it's the exact same positive_section instance.
            let positive_section_ptr = &format.positive_section as *const FormatSection;
            let selected_section_ptr = section_to_use as *const FormatSection;

            if positive_section_ptr == selected_section_ptr {
                // This is the Excel-like fallback scenario.
                is_fallback_for_negative = true;
                // In fallback, formatting rules (placeholders, scaling) apply to the absolute value.
                value_for_formatting_placeholders = value.abs();
            }
            // If a conditional section (not positive_section) was chosen for the negative value,
            // or if some other logic in select_section led to a different section,
            // it's not the simple P(abs(V)) fallback. is_fallback_for_negative remains false.
        }
        // If format.negative_section IS defined and was chosen by select_section,
        // is_fallback_for_negative remains false.
        // value_for_formatting_placeholders will be the original negative 'value'.
        // core::format_value's internal abs_value_for_formatting will handle abs() on it,
        // while original_value_for_sign_check (also negative) correctly sets is_negative.
    }
    // If value is 0.0 and zero_section is picked, or value is positive and positive_section is picked,
    // is_fallback_for_negative remains false, and value_for_formatting_placeholders is the original value.

    core::format_value(
        original_value_for_sign_check,
        value_for_formatting_placeholders,
        section_to_use,
        locale,
        is_fallback_for_negative,
    )
}
