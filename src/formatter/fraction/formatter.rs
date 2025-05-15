use super::placeholder_handler::format_integer_like_segment;
use super::utils;
use crate::types::{FormatSection, FormatToken, LocaleSettings};

#[derive(Debug, Default, Clone)]
pub struct FractionPatternAnalysis {
    pub is_fraction_format: bool,
    pub integer_part_tokens: Vec<FormatToken>,
    pub numerator_tokens: Vec<FormatToken>,
    pub denominator_tokens: Vec<FormatToken>, // For '/' based denominators
    pub fixed_denominator_value: Option<u32>, // From section.fixed_denominator
    pub has_explicit_slash: bool, // True if a '/' token exists (not a fixed denominator like #/16 where slash is implicit)
}

pub fn analyze_fraction_pattern(section: &FormatSection) -> FractionPatternAnalysis {
    let mut analysis = FractionPatternAnalysis {
        fixed_denominator_value: section.fixed_denominator,
        ..Default::default()
    };

    if section.fixed_denominator.is_some() {
        analysis.is_fraction_format = true;
        analysis.has_explicit_slash = false;

        let mut current_segment: Vec<FormatToken> = Vec::new();
        let mut segments: Vec<Vec<FormatToken>> = Vec::new();
        let mut integer_candidate_tokens: Vec<FormatToken> = Vec::new();
        let mut last_was_placeholder = false;

        for token in section.tokens.iter().cloned() {
            match token {
                FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace => {
                    current_segment.push(token.clone());
                    last_was_placeholder = true;
                }
                _ => {
                    if last_was_placeholder && !current_segment.is_empty() {
                        segments.push(current_segment);
                        current_segment = Vec::new();
                    }
                    if !segments.is_empty() {
                        segments.last_mut().unwrap().push(token);
                    } else {
                        integer_candidate_tokens.push(token);
                    }
                    last_was_placeholder = false;
                }
            }
        }
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        if segments.is_empty() {
            analysis.integer_part_tokens = integer_candidate_tokens;
        } else {
            analysis.numerator_tokens = segments.pop().unwrap_or_default();
            analysis.integer_part_tokens = integer_candidate_tokens;
            for seg in segments {
                analysis.integer_part_tokens.extend(seg);
            }
        }
    } else {
        let slash_pos = section
            .tokens
            .iter()
            .position(|t| matches!(t, FormatToken::LiteralChar('/')));

        if let Some(pos) = slash_pos {
            analysis.is_fraction_format = true;
            analysis.has_explicit_slash = true;

            let before_slash_tokens = &section.tokens[..pos];
            let after_slash_tokens = &section.tokens[pos + 1..];

            for token in after_slash_tokens.iter().cloned() {
                match token {
                    FormatToken::DigitOrZero
                    | FormatToken::DigitIfNeeded
                    | FormatToken::DigitOrSpace => {
                        analysis.denominator_tokens.push(token);
                    }
                    _ => break,
                }
            }

            let mut current_segment: Vec<FormatToken> = Vec::new();
            let mut segments: Vec<Vec<FormatToken>> = Vec::new();
            let mut integer_candidate_tokens: Vec<FormatToken> = Vec::new();
            let mut last_was_placeholder = false;

            for token in before_slash_tokens.iter().cloned() {
                match token {
                    FormatToken::DigitOrZero
                    | FormatToken::DigitIfNeeded
                    | FormatToken::DigitOrSpace => {
                        current_segment.push(token.clone());
                        last_was_placeholder = true;
                    }
                    _ => {
                        if last_was_placeholder && !current_segment.is_empty() {
                            segments.push(current_segment);
                            current_segment = Vec::new();
                        }
                        if !segments.is_empty() {
                            segments.last_mut().unwrap().push(token);
                        } else {
                            integer_candidate_tokens.push(token);
                        }
                        last_was_placeholder = false;
                    }
                }
            }
            if !current_segment.is_empty() {
                segments.push(current_segment);
            }

            if segments.is_empty() {
                analysis.integer_part_tokens = integer_candidate_tokens;
            } else {
                analysis.numerator_tokens = segments.pop().unwrap_or_default();
                analysis.integer_part_tokens = integer_candidate_tokens;
                for seg in segments {
                    analysis.integer_part_tokens.extend(seg);
                }
            }
        }
    }
    analysis
}

#[allow(clippy::too_many_arguments)]
pub fn format_number_as_fraction(
    original_value_for_sign: f64,
    value_for_formatting_placeholders: f64,
    _locale: &LocaleSettings,
    integer_part_tokens: &[FormatToken],
    numerator_tokens: &[FormatToken],
    denominator_tokens: &[FormatToken],
    fixed_denominator_value: Option<u32>,
    has_explicit_slash: bool,
    section_has_only_placeholders: bool,
) -> Option<String> {
    let abs_value = value_for_formatting_placeholders;
    let integer_part_val_f = abs_value.trunc();
    let mut decimal_part = abs_value.fract();
    if decimal_part < 0.0 {
        decimal_part = decimal_part.abs();
    }

    let (mut num_val, den_val): (i64, i64) = if let Some(fixed_den) = fixed_denominator_value {
        if fixed_den == 0 {
            return None;
        }
        (
            (decimal_part * fixed_den as f64).round() as i64,
            fixed_den as i64,
        )
    } else {
        if denominator_tokens.is_empty() {
            return None;
        }
        let max_den_precision = denominator_tokens.len().max(1);
        utils::decimal_to_fraction(decimal_part, max_den_precision)?
    };

    let mut final_integer_val_i64 = integer_part_val_f as i64;
    if num_val == den_val && den_val != 0 {
        if abs_value >= 0.0 {
            final_integer_val_i64 += 1;
        } else {
            final_integer_val_i64 -= 1;
        }
        num_val = 0;
    } else if num_val > den_val && den_val != 0 {
        if abs_value >= 0.0 {
            final_integer_val_i64 += num_val / den_val;
        } else {
            final_integer_val_i64 -= num_val / den_val;
        }
        num_val %= den_val;
    }

    let show_leading_sign = original_value_for_sign < 0.0;
    let int_digits_str = final_integer_val_i64.abs().to_string();
    let int_segment_is_effectively_zero = final_integer_val_i64 == 0;

    let mut int_part_formatted = if integer_part_tokens.is_empty() {
        String::new()
    } else {
        format_integer_like_segment(
            &int_digits_str,
            integer_part_tokens,
            int_segment_is_effectively_zero,
        )
    };

    let mut display_int_part = !int_part_formatted.is_empty()
        || (final_integer_val_i64 == 0
            && num_val == 0
            && integer_part_tokens
                .iter()
                .any(|t| matches!(t, FormatToken::DigitOrZero)));

    if final_integer_val_i64 == 0
        && num_val != 0
        && integer_part_tokens.len() == 1
        && matches!(integer_part_tokens[0], FormatToken::DigitOrZero)
    {
        int_part_formatted = " ".to_string();
        display_int_part = true;
    }

    let mut only_hash_and_spaces_in_int_tokens = false;
    let non_literal_int_tokens: Vec<&FormatToken> = integer_part_tokens
        .iter()
        .filter(|t| !matches!(t, FormatToken::LiteralChar(_)))
        .collect();
    if non_literal_int_tokens.len() == 1
        && matches!(non_literal_int_tokens[0], FormatToken::DigitIfNeeded)
        && integer_part_tokens.iter().all(|t| {
            matches!(t, FormatToken::DigitIfNeeded) || matches!(t, FormatToken::LiteralChar(' '))
        })
    {
        only_hash_and_spaces_in_int_tokens = true;
    }

    if final_integer_val_i64 == 0
        && num_val == 0
        && only_hash_and_spaces_in_int_tokens
        && section_has_only_placeholders
    {
        int_part_formatted = "0".to_string();
        display_int_part = true;
    }

    if final_integer_val_i64 == 0
        && integer_part_tokens.len() == 1
        && matches!(integer_part_tokens[0], FormatToken::DigitOrZero)
        && section_has_only_placeholders
        && (!numerator_tokens.is_empty()
            || !denominator_tokens.is_empty()
            || fixed_denominator_value.is_some())
        && int_part_formatted.trim().is_empty()
    {
        display_int_part = false;
    }

    let mut parts: Vec<String> = Vec::new();
    if display_int_part {
        parts.push(int_part_formatted.clone());
    }

    let mut result_str;

    let mut force_display_fraction_as_zero_denom = false;
    if value_for_formatting_placeholders == 0.0
        && section_has_only_placeholders
        && integer_part_tokens.is_empty()
        && (fixed_denominator_value.is_some() || !denominator_tokens.is_empty())
    {
        force_display_fraction_as_zero_denom = true;
    }

    if !force_display_fraction_as_zero_denom && num_val == 0 {
        if section_has_only_placeholders {
            let mut fraction_spaces_vec: Vec<String> = Vec::new();
            fraction_spaces_vec
                .extend(std::iter::repeat_n(" ".to_string(), numerator_tokens.len()));
            if (has_explicit_slash || fixed_denominator_value.is_some())
                && (!numerator_tokens.is_empty()
                    || !denominator_tokens.is_empty()
                    || fixed_denominator_value.is_some())
            {
                fraction_spaces_vec.push(" ".to_string());
            }
            fraction_spaces_vec.extend(std::iter::repeat_n(
                " ".to_string(),
                denominator_tokens.len(),
            ));

            parts.extend(fraction_spaces_vec);

            result_str = parts.join("");
        } else if parts.is_empty() {
            result_str = "0".to_string();
        } else {
            result_str = parts.join("");
        }
    } else {
        let current_num_val = if force_display_fraction_as_zero_denom {
            0
        } else {
            num_val
        };
        let current_den_val = if force_display_fraction_as_zero_denom {
            if den_val == 0 { 1 } else { den_val }
        } else {
            den_val
        };

        let formatted_numerator =
            format_integer_like_segment(&current_num_val.to_string(), numerator_tokens, false);

        if display_int_part {
            if !int_part_formatted.is_empty() && !int_part_formatted.ends_with(' ') {
                parts.push(" ".to_string());
            }
            parts.push(formatted_numerator);
        } else {
            if final_integer_val_i64 == 0
                && integer_part_tokens
                    .iter()
                    .any(|t| matches!(t, FormatToken::DigitIfNeeded))
                && !formatted_numerator.starts_with(' ')
                && !numerator_tokens.is_empty()
            {
                parts.push(" ".to_string());
            }
            parts.push(formatted_numerator);
        }

        if has_explicit_slash || fixed_denominator_value.is_some() {
            parts.push("/".to_string());
        }

        let den_digits_str = current_den_val.to_string();
        let den_fmt_raw = if fixed_denominator_value.is_some() {
            den_digits_str
        } else {
            format_integer_like_segment(&den_digits_str, denominator_tokens, false)
        };

        let den_fmt_final = if fixed_denominator_value.is_none() && !denominator_tokens.is_empty() {
            let trimmed_start = den_fmt_raw.trim_start();
            format!(
                "{}{}",
                trimmed_start,
                " ".repeat(den_fmt_raw.len() - trimmed_start.len())
            )
        } else {
            den_fmt_raw
        };
        parts.push(den_fmt_final);
        result_str = parts.join("");
    }

    if show_leading_sign
        && !result_str.is_empty()
        && result_str.trim() != "0"
        && !result_str.starts_with('-')
    {
        result_str.insert(0, '-');
    }
    if result_str.is_empty() && final_integer_val_i64 == 0 && num_val == 0 {
        return Some("0".to_string());
    }

    Some(result_str)
}
