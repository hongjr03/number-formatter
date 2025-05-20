pub mod parser;
pub use parser::parse_number_format;
pub mod formatter;
pub mod locale;
pub mod types;

// Re-export commonly used locale functions
pub use locale::{get_locale_settings, get_locale_settings_by_code};

use types::LocaleSettings;
/// Formats a number according to a parsed format string and locale settings.
///
/// # Examples
/// ```
/// use number_format::{parse_number_format, format_number};
/// use number_format::types::LocaleSettings;
///
/// let fmt = parse_number_format("#,##0.00").unwrap();
/// let num = 12345.678;
/// let formatted_default_locale = format_number(num, &fmt, &LocaleSettings::default());
/// assert_eq!(formatted_default_locale, "12,345.68"); // Assuming thousands separator is implemented
///
/// let german_locale = LocaleSettings::default()
///     .with_decimal_point(',')
///     .with_thousands_separator('.');
/// let formatted_german_locale = format_number(num, &fmt, &german_locale);
/// // Expected: "12.345,68" (once thousands separator is implemented and respecting locale)
/// // For now, without thousands separator: "12345,68"
/// assert_eq!(formatted_german_locale, "12.345,68"); // Update this line
/// ```
pub fn format_number(value: f64, format: &types::NumberFormat, locale: &LocaleSettings) -> String {
    formatter::format_number(value, format, locale)
}
