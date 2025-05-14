use crate::types::{ExponentialNotation, FormatSection, FormatToken, LocaleSettings};
use std::fmt::Write;

/// Format a number in exponential notation
pub(super) fn format_exponential(
    value: f64,
    section: &FormatSection,
    exp_token_idx: usize,
    locale: &LocaleSettings,
) -> String {
    let mut result = String::new();

    // Get the exponential token
    let exp_token = &section.tokens[exp_token_idx];
    let _exp_notation_type = match exp_token {
        // Renamed to avoid unused var warning if only sign matters
        FormatToken::Exponential(notation) => notation,
        _ => unreachable!(), // Should be caught by caller
    };

    // Format with scientific notation
    let abs_value = value.abs();
    let (mantissa, exponent) = if abs_value == 0.0 {
        (0.0, 0)
    } else {
        let log10_val = abs_value.log10();
        let exponent_val = log10_val.floor();
        let mantissa_val = abs_value / 10.0_f64.powf(exponent_val);
        (mantissa_val, exponent_val as i32)
    };

    // Format mantissa part with proper precision
    let is_negative = value < 0.0;
    let sign = if is_negative { "-" } else { "" };

    // Count number of desired decimal places in mantissa
    let mut mantissa_precision = 0; // Default to 0, meaning only the integer part of mantissa if no frac part in format
    let mut in_mantissa_decimal_part = false;
    for token in section.tokens.iter().take(exp_token_idx) {
        if matches!(token, FormatToken::DecimalPoint) {
            in_mantissa_decimal_part = true;
            continue;
        }
        if in_mantissa_decimal_part
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            mantissa_precision += 1;
        }
    }
    // If no decimal point was found before E, check if there are integer placeholders before E
    // Excel default: For "0E+00", 12345 -> 1E+04 (no decimals in mantissa)
    // For "0.00E+00", 12345 -> 1.23E+04 (2 decimals in mantissa)
    // So, mantissa_precision calculation above based on tokens after decimal point and before E is correct.
    // If mantissa_precision is still 0, it means format like "0E+00".

    // Round mantissa correctly based on its desired precision
    let power = 10.0_f64.powi(mantissa_precision as i32);
    let rounded_mantissa = (mantissa * power).round() / power;

    // Adjust exponent if rounding mantissa caused it to become >= 10 or < 1
    let (final_mantissa, final_exponent) = if rounded_mantissa == 0.0 {
        // handle 0.0 case separately
        (0.0, 0)
    } else if rounded_mantissa >= 10.0 {
        (rounded_mantissa / 10.0, exponent + 1)
    } else if rounded_mantissa < 1.0 && mantissa != 0.0 {
        // mantissa != 0.0 to avoid 0.0 becoming 0.0 E-1
        // This case needs care: if format is 0E+00, and value is 0.123 -> 1E-01
        // if format is 0.0E+00, and value is 0.0123 -> 1.2E-02
        // The initial mantissa calculation (abs_value / 10.0_f64.powf(exponent_val)) ensures mantissa >=1 and <10
        // So, rounding alone should not make it < 1 unless original value was very small and precision is low.
        // If it does become < 1 due to rounding (e.g. 1.0000xxx rounded to 0 precision -> 1.0, but if 0.5 rounded to 0 precision -> 1.0. If 0.4 -> 0.0)
        // Let's stick to initial mantissa/exponent adjustment and rely on precision for rounding.
        (rounded_mantissa, exponent) // Revisit if exponent adjustment for mantissa < 1 due to rounding is needed
    } else {
        (rounded_mantissa, exponent)
    };

    write!(result, "{}", sign).unwrap();

    let mut mantissa_str = format!(
        "{:.precision$}",
        final_mantissa,
        precision = mantissa_precision
    );
    if locale.decimal_point != '.' {
        mantissa_str = mantissa_str.replace('.', &locale.decimal_point.to_string());
    }
    write!(result, "{}", mantissa_str).unwrap();

    // Add E notation
    let final_exp_sign_str = if final_exponent < 0 {
        "-"
    } else {
        match &section.tokens[exp_token_idx] {
            FormatToken::Exponential(ExponentialNotation::Plus) => "+",
            FormatToken::Exponential(ExponentialNotation::Minus) => "",
            _ => unreachable!(), // Should be caught by caller or parser
        }
    };
    write!(result, "E{}{:02}", final_exp_sign_str, final_exponent.abs()).unwrap();

    result
}
