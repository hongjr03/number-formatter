//! Locale support for number formatting
//!
//! This module handles loading and managing locale-specific settings
//! for number and date formatting based on locale identifiers.

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use crate::types::LocaleSettings;

/// Error type for locale operations
#[derive(Debug, Clone, PartialEq)]
pub enum LocaleError {
    /// The specified locale was not found
    NotFound(String),
    /// An error occurred while parsing locale data
    ParseError(String),
}

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocaleError::NotFound(locale) => write!(f, "Locale not found: {}", locale),
            LocaleError::ParseError(msg) => write!(f, "Error parsing locale data: {}", msg),
        }
    }
}

impl std::error::Error for LocaleError {}

type Result<T> = std::result::Result<T, LocaleError>;

/// Represents a locale manager that provides access to locale-specific settings
pub struct LocaleManager {
    locale_codes: HashMap<u32, String>,
    locale_settings: HashMap<String, LocaleSettings>,
}

// Global singleton for locale settings
static LOCALE_MANAGER: OnceLock<LocaleManager> = OnceLock::new();

impl LocaleManager {
    /// Create a new locale manager with the default locale data
    fn new() -> Self {
        let mut manager = Self {
            locale_codes: HashMap::new(),
            locale_settings: HashMap::new(),
        };

        // Parse and load the built-in locale data
        if let Err(e) = manager.load_embedded_data() {
            // Just log the error and continue with empty maps
            eprintln!("Failed to load embedded locale data: {}", e);
        }

        manager
    }

    /// Load the embedded locale data from the TOML files
    fn load_embedded_data(&mut self) -> Result<()> {
        // Load locale code mapping
        let locale_codes_toml = include_str!("locale/locale_codes.toml");
        self.parse_locale_codes(locale_codes_toml)?;

        // Load locale settings
        let locale_settings_toml = include_str!("locale/locale_settings.toml");
        self.parse_locale_settings(locale_settings_toml)?;

        Ok(())
    }

    /// Parse the locale codes TOML data
    fn parse_locale_codes(&mut self, toml_str: &str) -> Result<()> {
        let parsed_toml: toml::Value =
            toml::from_str(toml_str).map_err(|e| LocaleError::ParseError(e.to_string()))?;

        let table = parsed_toml
            .as_table()
            .ok_or_else(|| LocaleError::ParseError("Root is not a table".to_string()))?;

        for (key, value) in table {
            if key.starts_with("code_") {
                let code_table = value
                    .as_table()
                    .ok_or_else(|| LocaleError::ParseError(format!("{} is not a table", key)))?;

                let code = code_table
                    .get("code")
                    .and_then(|v| v.as_integer())
                    .ok_or_else(|| {
                        LocaleError::ParseError(format!("Missing or invalid code in {}", key))
                    })?;

                let locale = code_table
                    .get("locale")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        LocaleError::ParseError(format!("Missing or invalid locale in {}", key))
                    })?;

                self.locale_codes.insert(code as u32, locale.to_string());
            }
        }

        Ok(())
    }

    /// Parse the locale settings TOML data
    fn parse_locale_settings(&mut self, toml_str: &str) -> Result<()> {
        let parsed_toml: toml::Value =
            toml::from_str(toml_str).map_err(|e| LocaleError::ParseError(e.to_string()))?;

        let table = parsed_toml
            .as_table()
            .ok_or_else(|| LocaleError::ParseError("Root is not a table".to_string()))?;

        // First load base settings if available
        let base_settings = if let Some(base) = table.get("base") {
            self.parse_locale_setting(base)?
        } else {
            LocaleSettings::default()
        };

        // Now load each locale's settings
        for (locale_id, value) in table {
            if locale_id == "base" {
                continue; // Already handled
            }

            let mut locale_settings = base_settings.clone();

            // Apply locale-specific settings over the base settings
            self.apply_locale_specific_settings(&mut locale_settings, value)?;

            // Add to the map
            self.locale_settings
                .insert(locale_id.to_string(), locale_settings);
        }

        Ok(())
    }

    /// Parse a single locale setting from TOML
    fn parse_locale_setting(&self, value: &toml::Value) -> Result<LocaleSettings> {
        let table = value
            .as_table()
            .ok_or_else(|| LocaleError::ParseError("Locale setting is not a table".to_string()))?;

        let mut settings = LocaleSettings::default();

        // Decimal point
        if let Some(decimal) = table.get("decimal").and_then(|v| v.as_str()) {
            if let Some(c) = decimal.chars().next() {
                settings.decimal_point = c;
            }
        }

        // Thousands separator
        if let Some(group) = table.get("group").and_then(|v| v.as_str()) {
            if let Some(c) = group.chars().next() {
                settings.thousands_separator = c;
            }
        }

        // AM/PM markers
        if let Some(ampm) = table.get("ampm").and_then(|v| v.as_array()) {
            if ampm.len() >= 2 {
                let am = ampm[0].as_str().unwrap_or("AM").to_string();
                let pm = ampm[1].as_str().unwrap_or("PM").to_string();
                settings.ampm_markers = [am, pm];
            }
        }

        // Month names (full)
        if let Some(months) = table.get("month_names").and_then(|v| v.as_array()) {
            if months.len() == 12 {
                let month_names: Vec<String> = months
                    .iter()
                    .map(|m| m.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = month_names.clone().try_into() {
                    settings.month_names = array;
                }
                settings.month_names_full = month_names;
            }
        }

        // Month abbreviations
        if let Some(months) = table.get("month_abbreviations").and_then(|v| v.as_array()) {
            if months.len() == 12 {
                let month_abbrs: Vec<String> = months
                    .iter()
                    .map(|m| m.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = month_abbrs.clone().try_into() {
                    settings.short_month_names = array;
                }
                settings.month_names_abbr = month_abbrs;
            }
        }

        // Day names (full)
        if let Some(days) = table.get("day_names").and_then(|v| v.as_array()) {
            if days.len() == 7 {
                let day_names: Vec<String> = days
                    .iter()
                    .map(|d| d.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = day_names.try_into() {
                    settings.day_names = array;
                }
            }
        }

        // Day abbreviations
        if let Some(days) = table.get("day_abbreviations").and_then(|v| v.as_array()) {
            if days.len() == 7 {
                let day_abbrs: Vec<String> = days
                    .iter()
                    .map(|d| d.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = day_abbrs.try_into() {
                    settings.short_day_names = array;
                }
            }
        }

        Ok(settings)
    }

    /// Apply locale-specific settings from TOML to a LocaleSettings object
    fn apply_locale_specific_settings(
        &self,
        settings: &mut LocaleSettings,
        value: &toml::Value,
    ) -> Result<()> {
        let table = value
            .as_table()
            .ok_or_else(|| LocaleError::ParseError("Locale setting is not a table".to_string()))?;

        // Decimal point
        if let Some(decimal) = table.get("decimal").and_then(|v| v.as_str()) {
            if let Some(c) = decimal.chars().next() {
                settings.decimal_point = c;
            }
        }

        // Thousands separator
        if let Some(group) = table.get("group").and_then(|v| v.as_str()) {
            if let Some(c) = group.chars().next() {
                settings.thousands_separator = c;
            }
        }

        // Currency symbol (if applicable)
        if let Some(currency) = table.get("currency_symbol").and_then(|v| v.as_str()) {
            settings.currency_symbol = currency.to_string();
        } else {
            // If the locale includes a country code, try to determine a default currency symbol
            // This is a very simplified approach and should be replaced with proper data
            settings.currency_symbol = "$".to_string(); // Default
        }

        // AM/PM markers
        if let Some(ampm) = table.get("ampm").and_then(|v| v.as_array()) {
            if ampm.len() >= 2 {
                let am = ampm[0].as_str().unwrap_or("AM").to_string();
                let pm = ampm[1].as_str().unwrap_or("PM").to_string();
                settings.ampm_markers = [am, pm];
            }
        }

        // Month names (full)
        if let Some(months) = table.get("month_names").and_then(|v| v.as_array()) {
            if months.len() == 12 {
                let month_names: Vec<String> = months
                    .iter()
                    .map(|m| m.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = month_names.clone().try_into() {
                    settings.month_names = array;
                }
                settings.month_names_full = month_names;
            }
        }

        // Month abbreviations
        if let Some(months) = table.get("month_abbreviations").and_then(|v| v.as_array()) {
            if months.len() == 12 {
                let month_abbrs: Vec<String> = months
                    .iter()
                    .map(|m| m.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = month_abbrs.clone().try_into() {
                    settings.short_month_names = array;
                }
                settings.month_names_abbr = month_abbrs;
            }
        }

        // Day names (full)
        if let Some(days) = table.get("day_names").and_then(|v| v.as_array()) {
            if days.len() == 7 {
                let day_names: Vec<String> = days
                    .iter()
                    .map(|d| d.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = day_names.try_into() {
                    settings.day_names = array;
                }
            }
        }

        // Day abbreviations
        if let Some(days) = table.get("day_abbreviations").and_then(|v| v.as_array()) {
            if days.len() == 7 {
                let day_abbrs: Vec<String> = days
                    .iter()
                    .map(|d| d.as_str().unwrap_or("").to_string())
                    .collect();

                if let Ok(array) = day_abbrs.try_into() {
                    settings.short_day_names = array;
                }
            }
        }

        Ok(())
    }

    /// Get the global locale manager instance
    fn get() -> &'static Self {
        LOCALE_MANAGER.get_or_init(Self::new)
    }

    /// Get locale settings by locale identifier (e.g., "en_US", "zh_CN")
    fn get_locale_settings(&self, locale_id: &str) -> Option<&LocaleSettings> {
        self.locale_settings.get(locale_id)
    }

    /// Resolve a locale code (numeric) to a locale identifier
    fn resolve_locale_code(&self, code: u32) -> Option<&str> {
        self.locale_codes.get(&code).map(|s| s.as_str())
    }
}

/// Get locale settings by locale identifier (e.g., "en_US", "zh_CN")
pub fn get_locale_settings(locale_id: &str) -> Option<LocaleSettings> {
    LocaleManager::get().get_locale_settings(locale_id).cloned()
}

/// Get locale settings by Excel-style locale code (e.g., 0x0409 for en_US)
pub fn get_locale_settings_by_code(code: u32) -> Option<LocaleSettings> {
    let manager = LocaleManager::get();
    manager
        .resolve_locale_code(code)
        .and_then(|locale_id| manager.get_locale_settings(locale_id))
        .cloned()
}

/// Get locale settings for the [$-XXXX] format specifier in Excel
pub fn get_locale_settings_for_excel_code(code_str: &str) -> Option<LocaleSettings> {
    // Excel format is typically [$-409] or similar
    // Extract the numeric part and convert to a code
    if let Some(code_part) = code_str
        .strip_prefix("[$-")
        .and_then(|s| s.strip_suffix("]"))
    {
        // Try to parse as hex
        if let Ok(code) = u32::from_str_radix(code_part, 16) {
            return get_locale_settings_by_code(code);
        }

        // Try to parse as decimal
        if let Ok(code) = code_part.parse::<u32>() {
            return get_locale_settings_by_code(code);
        }

        // Check if it's a direct locale name like "zh-TW"
        let normalized = code_part.replace('-', "_");
        return get_locale_settings(&normalized);
    }

    None
}

/// Get locale settings for a prefix like "[$US-409]"
pub fn get_locale_settings_with_prefix(prefix: &str, code_str: &str) -> Option<LocaleSettings> {
    let mut settings = get_locale_settings_for_excel_code(code_str)?;

    // Update the currency symbol based on the prefix
    // This is a simple implementation - in reality you might want to look up
    // appropriate currency symbols for specific prefixes
    settings.currency_symbol = prefix.to_string();

    Some(settings)
}

/// List all available locale identifiers
pub fn list_available_locales() -> Vec<String> {
    LocaleManager::get()
        .locale_settings
        .keys()
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_loading() {
        // Ensure locale data is loaded
        let locales = list_available_locales();
        assert!(!locales.is_empty(), "Should have loaded some locales");

        // Check some common locales
        let en_us = get_locale_settings("en_US");
        assert!(en_us.is_some(), "Should have en_US locale");

        if let Some(settings) = en_us {
            assert_eq!(settings.decimal_point, '.');
            assert_eq!(settings.thousands_separator, ',');
        }
    }

    #[test]
    fn test_locale_code_resolution() {
        // Test US English (0x409)
        let en_us = get_locale_settings_by_code(0x409);
        assert!(en_us.is_some(), "Should resolve locale code 0x409 to en_US");

        // Test Chinese (0x804)
        let zh_cn = get_locale_settings_by_code(0x804);
        assert!(zh_cn.is_some(), "Should resolve locale code 0x804 to zh_CN");
    }

    #[test]
    fn test_excel_code_format() {
        // Test with Excel format [$-409]
        let en_us = get_locale_settings_for_excel_code("[$-409]");
        assert!(en_us.is_some(), "Should parse Excel format [$-409]");

        // Test with prefix [$US-409]
        let us_dollar = get_locale_settings_with_prefix("US", "[$-409]");
        assert!(us_dollar.is_some(), "Should parse format with prefix");
        if let Some(settings) = us_dollar {
            assert_eq!(settings.currency_symbol, "US");
        }
    }
}
