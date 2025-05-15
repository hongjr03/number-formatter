use crate::types::FormatToken;

/// Formats an integer-like numeric string segment (e.g., integer part, numerator, denominator)
/// based on a series of `0`, `#`, or `?` placeholders.
///
/// Args:
/// * `digits_str`: The string of digits to format (e.g., "123", "0", "7").
/// * `placeholders`: A slice of `FormatToken`s, expected to be `Zero`, `Hash`, or `Question`.
/// * `actual_value_is_zero`: True if the original numeric value this `digits_str` represents was zero.
///   This is important for the behavior of `#` with a zero value (results in an empty string).
///
/// Returns:
/// A string with the digits formatted according to the placeholders.
pub fn format_integer_like_segment(
    digits_str: &str,
    placeholders: &[FormatToken],
    actual_value_is_zero: bool,
) -> String {
    if actual_value_is_zero &&
       digits_str.chars().all(|c| c == '0') && // e.g. "0", "00"
       placeholders.iter().all(|p| matches!(p, FormatToken::DigitIfNeeded))
    {
        return "".to_string();
    }

    let n_place = placeholders.len();
    // Sentinel for unassigned placeholder slot, or a slot taken by Hash that got no digit.
    const EMPTY_SLOT: char = '\0';
    let mut output_slots: Vec<char> = vec![EMPTY_SLOT; n_place];

    let effective_digits_str: String;
    if actual_value_is_zero {
        effective_digits_str = "0".to_string();
    } else {
        let temp_trimmed = digits_str.trim_start_matches('0');
        if temp_trimmed.is_empty() {
            // Original was "0" or "00..."
            effective_digits_str = "0".to_string(); // e.g. integer part of 0.xxx
        } else {
            effective_digits_str = temp_trimmed.to_string(); // e.g. "007" became "7"
        }
    }
    let mut digit_chars: Vec<char> = effective_digits_str.chars().collect(); // Treated as a stack for popping from the right (least significant digit)

    // Pass 1: Fill '0' and '#' placeholders from right to left (of placeholders array)
    // Consumes digits from the right from digit_chars (least significant first)
    for i in (0..n_place).rev() {
        match placeholders[i] {
            FormatToken::DigitOrZero => {
                if let Some(digit) = digit_chars.pop() {
                    output_slots[i] = digit;
                } else {
                    output_slots[i] = '0';
                }
            }
            FormatToken::DigitIfNeeded => {
                if let Some(digit) = digit_chars.pop() {
                    // If the overall value is zero, and this digit is '0',
                    // and it's the last digit from the (normalized to "0") input,
                    // then this # should render as empty.
                    if actual_value_is_zero && digit == '0' && digit_chars.is_empty() {
                        // Stays EMPTY_SLOT, will be skipped in final assembly
                    } else {
                        output_slots[i] = digit;
                    }
                } else {
                    // Stays EMPTY_SLOT, will be skipped in final assembly
                }
            }
            FormatToken::DigitOrSpace => {
                // '?' are handled in Pass 2
            }
            _ => { /* Non-placeholder token, should ideally not be in `placeholders` input */ }
        }
    }

    // Pass 2: Fill '?' placeholders from right to left (of placeholders array)
    // Consumes any further digits from digit_chars. If no digits left, '?' becomes a space.
    for i in (0..n_place).rev() {
        if matches!(placeholders[i], FormatToken::DigitOrSpace) {
            // This slot would not have been filled by Pass 1 (which only handles Zero and Hash)
            // So, output_slots[i] should still be EMPTY_SLOT here if it's a Question mark.
            if let Some(digit) = digit_chars.pop() {
                output_slots[i] = digit;
            } else {
                output_slots[i] = ' '; // No digit for this '?', so it's a space
            }
        }
    }

    let mut final_result: String = String::new();
    // Prepend any digits that were more significant than all placeholders
    // (i.e., digit_chars still has items after trying to fill all placeholders)
    while let Some(digit) = digit_chars.pop() {
        final_result.insert(0, digit); // insert at the beginning to maintain order
    }

    // Assemble the string from output_slots, respecting placeholder types
    let part_from_placeholders: String =
        output_slots.iter().filter(|&&c| c != EMPTY_SLOT).collect();
    final_result.push_str(&part_from_placeholders);

    final_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FormatToken::{DigitIfNeeded as H, DigitOrSpace as Q, DigitOrZero as Z};

    #[test]
    fn test_format_integer_segment_simple_zero() {
        assert_eq!(
            format_integer_like_segment("123", &[Z, Z, Z, Z, Z], false),
            "00123"
        );
        assert_eq!(
            format_integer_like_segment("12", &[Z, Z, Z, Z], false),
            "0012"
        );
    }

    #[test]
    fn test_format_integer_segment_simple_hash() {
        assert_eq!(
            format_integer_like_segment("123", &[H, H, H, H, H], false),
            "123"
        );
        assert_eq!(format_integer_like_segment("12", &[H, H], false), "12");
        assert_eq!(
            format_integer_like_segment("12", &[H, H, H, H], false),
            "12"
        );
    }

    #[test]
    fn test_format_integer_segment_value_longer_than_placeholders() {
        assert_eq!(
            format_integer_like_segment("12345", &[Z, Z, Z], false),
            "12345"
        );
        assert_eq!(
            format_integer_like_segment("12345", &[H, H, H], false),
            "12345"
        );
        assert_eq!(
            format_integer_like_segment("12345", &[Q, Q, Q], false),
            "12345"
        );
    }

    #[test]
    fn test_format_integer_segment_zero_value_with_hash() {
        assert_eq!(format_integer_like_segment("0", &[H], true), "");
        assert_eq!(format_integer_like_segment("0", &[H, H, H], true), "");
        assert_eq!(format_integer_like_segment("0", &[Z, H], true), "0"); // # is not all
    }

    #[test]
    fn test_format_integer_segment_simple_question() {
        assert_eq!(
            format_integer_like_segment("12", &[Q, Q, Q, Q], false),
            "  12"
        );
        assert_eq!(format_integer_like_segment("7", &[Q, Q, Q], false), "  7");
    }

    #[test]
    fn test_format_integer_segment_zero_value_with_question() {
        assert_eq!(format_integer_like_segment("0", &[Q], true), "0");
        assert_eq!(format_integer_like_segment("0", &[Q, Q], true), " 0");
        assert_eq!(format_integer_like_segment("0", &[Q, Q, Q], true), "  0");
    }

    #[test]
    fn test_format_integer_segment_mixed_placeholders() {
        // Format "0#??", Value 7 -> "07  "
        assert_eq!(
            format_integer_like_segment("7", &[Z, H, Q, Q], false),
            "07  "
        );
        // Format "#0#", Value 0 -> "0" (middle 0 forces it)
        assert_eq!(format_integer_like_segment("0", &[H, Z, H], true), "0");
        // Format "??0", Value 7 -> "  7" (0 acts like # if digit present)
        assert_eq!(format_integer_like_segment("7", &[Q, Q, Z], false), "  7");
        // Format "??0", Value 0 -> "  0"
        assert_eq!(format_integer_like_segment("0", &[Q, Q, Z], true), "  0");
    }

    #[test]
    fn test_format_integer_segment_leading_zeros_in_digits() {
        assert_eq!(format_integer_like_segment("007", &[Z, Z, Z], false), "007");
        assert_eq!(format_integer_like_segment("007", &[H, H, H], false), "7"); // Standard # behavior
        assert_eq!(format_integer_like_segment("007", &[Q, Q, Q], false), "  7"); // Q should reflect significance
    }

    #[test]
    fn test_from_core_rs_logic_issues() {
        // core.rs old logic: format_padded_numerator("1", 1, 3, false) -> "  1" (num_val, num_q_count, den_q_count, is_int_zero_and_frac_zero)
        // This is for ?/? with value 1/x (num_q=1 for num, den_q=1 for den) -> " 1/X"
        // Here, digit_str="1", placeholders=[Q]. Should be "1".
        // The spaces are context-dependent (alignment with other parts of fraction).
        // This function `format_integer_like_segment` should just format the part.
        assert_eq!(format_integer_like_segment("1", &[Q], false), "1");

        // integer part "0" for # ?/? with value 0.5 -> " "
        // Here, digit_str="0", placeholders=[H]. actual_value_is_zero for this part is true. -> ""
        assert_eq!(format_integer_like_segment("0", &[H], true), "");
    }
}
