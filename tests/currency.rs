#[cfg(test)]
mod tests {
    use number_format::{
        format_number,                         // The public one from lib.rs
        parse_number_format,                   // The public one from lib.rs
        types::{LocaleSettings, NumberFormat}, // LocaleSettings and NumberFormat from types module
    };
    // NumberFormatError is not a type, parse_number_format returns Result<_, String>

    // Helper function to format with a specific locale and format string
    fn fmt_currency(
        value: f64,
        format_code: &str,
        locale: &LocaleSettings,
    ) -> Result<String, String> {
        let fmt: NumberFormat = parse_number_format(format_code)?;
        Ok(format_number(value, &fmt, locale))
    }

    #[test]
    fn test_locale_currency_symbol_euro() -> Result<(), String> {
        let euro_locale = LocaleSettings::default().with_currency_symbol("€".to_string());
        assert_eq!(
            fmt_currency(1234.56, "¤#,##0.00", &euro_locale)?,
            "€1,234.56"
        );
        assert_eq!(
            fmt_currency(-1234.56, "¤#,##0.00;(¤#,##0.00)", &euro_locale)?,
            "(€1,234.56)"
        );
        assert_eq!(fmt_currency(0.0, "¤0.00", &euro_locale)?, "€0.00");
        assert_eq!(fmt_currency(100.0, "0.00¤", &euro_locale)?, "100.00€");
        Ok(())
    }

    #[test]
    fn test_locale_currency_symbol_yen() -> Result<(), String> {
        let yen_locale = LocaleSettings::default().with_currency_symbol("¥".to_string());
        assert_eq!(fmt_currency(12345.0, "¤#,##0", &yen_locale)?, "¥12,345");
        assert_eq!(
            fmt_currency(-12345.0, "¤#,##0;(¤#,##0)", &yen_locale)?,
            "(¥12,345)"
        );
        Ok(())
    }

    #[test]
    fn test_locale_currency_symbol_with_text() -> Result<(), String> {
        let custom_locale = LocaleSettings::default()
            .with_currency_symbol("CUSTOM".to_string())
            .with_decimal_point(',');
        assert_eq!(
            fmt_currency(1.0, "\"Amount: \"¤0.00", &custom_locale)?,
            "Amount: CUSTOM1,00"
        );
        Ok(())
    }

    // #[test]
    // fn test_locale_currency_symbol_in_text_section() -> Result<(), String> {
    //     let euro_locale = LocaleSettings::default().with_currency_symbol("€".to_string());
    //     let fmt_euro = parse_number_format("#;#;#;\"Value: \" @ \" (\"¤\")\"")?;
    //     assert_eq!(
    //         format_number(f64::NAN, &fmt_euro, &euro_locale),
    //         "Value:  NaN  (€)"
    //     );
    //     Ok(())
    // }

    #[test]
    fn test_multiple_locale_currency_symbols() -> Result<(), String> {
        let chf_locale = LocaleSettings::default().with_currency_symbol("CHF".to_string());
        assert_eq!(
            fmt_currency(789.0, "¤ #,##0.00 ¤", &chf_locale)?,
            "CHF 789.00 CHF"
        );
        Ok(())
    }

    #[test]
    fn test_locale_currency_without_digits() -> Result<(), String> {
        let cad_locale = LocaleSettings::default().with_currency_symbol("CAD ".to_string()); // Note space
        assert_eq!(fmt_currency(0.0, "¤", &cad_locale)?, "CAD ");
        assert_eq!(fmt_currency(0.0, "\"Code: \"¤", &cad_locale)?, "Code: CAD ");
        Ok(())
    }
}
