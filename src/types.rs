//! Type definitions for number format parser
//!
//! This module defines the type system used to represent the parsed results of format strings.
//! Includes tokens, comparison operators, and format sections.

/// Represents a single format token parsed from the format string
#[derive(Debug, Clone, PartialEq)]
pub enum FormatToken {
    /// Number placeholder (0) that shows zero if no digit exists
    DigitOrZero,
    /// Number placeholder (#) that shows nothing if no digit exists
    DigitIfNeeded,
    /// Number placeholder (?) that shows a space if no digit exists
    DigitOrSpace,
    /// Decimal point (.)
    DecimalPoint,
    /// Thousands separator (,)
    ThousandsSeparator,
    /// Percentage symbol (%)
    Percentage,
    /// Exponential notation, such as E+ or E-
    Exponential(ExponentialNotation),
    /// Literal character to display directly
    LiteralChar(char),
    /// Fill character, * followed by a character
    Fill(char),
    /// Skip width, _ followed by a character
    SkipWidth(char),
    /// Quoted text, like "text"
    QuotedText(String),
    /// Text value placeholder (@)
    TextValue,

    /// Color indicator, like [Red], [Blue], etc.
    Color(ColorType),

    /// Two-digit year (yy)
    YearTwoDigit,
    /// Four-digit year (yyyy)
    YearFourDigit,
    /// Month number, e.g., January is 1 (m)
    MonthNum,
    /// Zero-padded month number, e.g., January is 01 (mm)
    MonthNumPadded,
    /// Month abbreviation, e.g., Jan (mmm)
    MonthAbbr,
    /// Full month name, e.g., January (mmmm)
    MonthFullName,
    /// Month initial letter, e.g., J (mmmmm)
    MonthLetter,
    /// Day number, 1-31 (d)
    DayNum,
    /// Zero-padded day number, 01-31 (dd)
    DayNumPadded,
    /// Weekday abbreviation, e.g., Mon (ddd)
    WeekdayAbbr,
    /// Full weekday name, e.g., Monday (dddd)
    WeekdayFullName,
    /// Hour in 12 or 24-hour format (h)
    Hour12Or24,
    /// Zero-padded hour in 12 or 24-hour format (hh)
    Hour12Or24Padded,
    /// Minute number (m), determined to be minutes from context
    MinuteNum,
    /// Zero-padded minute number (mm), determined to be minutes from context
    MinuteNumPadded,
    /// Second number (s)
    SecondNum,
    /// Zero-padded second number (ss)
    SecondNumPadded,
    /// AM/PM indicator
    AmPm,
    /// A/P indicator
    AP,
    /// Elapsed hours [h]
    ElapsedHours,
    /// Elapsed minutes [m]
    ElapsedMinutes,
    /// Elapsed seconds [s]
    ElapsedSeconds,

    /// Single m, might be month or minute, to be determined by context
    MonthOrMinute1,
    /// Double m, might be month or minute, to be determined by context
    MonthOrMinute2,
}

/// Represents color types
#[derive(Debug, Clone, PartialEq)]
pub enum ColorType {
    /// Red color
    Red,
    /// Green color
    Green,
    /// Blue color
    Blue,
    /// Magenta color
    Magenta,
    /// Cyan color
    Cyan,
    /// Yellow color
    Yellow,
    /// Black color
    Black,
    /// White color
    White,
}

impl FormatToken {
    /// Checks if the token is related to numbers or dates
    pub fn is_numeric_or_date(&self) -> bool {
        matches!(
            self,
            FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace
                | FormatToken::DecimalPoint
                | FormatToken::ThousandsSeparator
                | FormatToken::Percentage
                | FormatToken::Exponential(_)
                | FormatToken::YearTwoDigit
                | FormatToken::YearFourDigit
                | FormatToken::MonthNum
                | FormatToken::MonthNumPadded
                | FormatToken::MonthAbbr
                | FormatToken::MonthFullName
                | FormatToken::MonthLetter
                | FormatToken::MonthOrMinute1
                | FormatToken::MonthOrMinute2
                | FormatToken::DayNum
                | FormatToken::DayNumPadded
                | FormatToken::WeekdayAbbr
                | FormatToken::WeekdayFullName
                | FormatToken::Hour12Or24
                | FormatToken::Hour12Or24Padded
                | FormatToken::MinuteNum
                | FormatToken::MinuteNumPadded
                | FormatToken::SecondNum
                | FormatToken::SecondNumPadded
                | FormatToken::AmPm
                | FormatToken::AP
                | FormatToken::ElapsedHours
                | FormatToken::ElapsedMinutes
                | FormatToken::ElapsedSeconds
        )
    }
}

/// Type of exponential notation
#[derive(Debug, Clone, PartialEq)]
pub enum ExponentialNotation {
    /// E+ notation
    Plus,
    /// E- notation
    Minus,
}

/// Comparison operators for conditional formatting
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    /// Equal to (=)
    Eq,
    /// Greater than (>)
    Gt,
    /// Less than (<)
    Lt,
    /// Greater than or equal to (>=)
    Ge,
    /// Less than or equal to (<=)
    Le,
    /// Not equal to (<>)
    Ne,
}

/// Represents a format condition with an operator and a comparison value
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    /// The comparison operator
    pub operator: ComparisonOperator,
    /// The value to compare against
    pub value: f64,
}

/// Represents a section of the format string with optional condition and a sequence of tokens
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormatSection {
    /// The section's condition, if any
    pub condition: Option<Condition>,
    /// Sequence of format tokens
    pub tokens: Vec<FormatToken>,
    /// Indicates if this is the fourth section (text section)
    pub is_text_section: bool,
}

/// Represents a complete number format with all sections
#[derive(Debug, Clone, PartialEq)]
pub struct NumberFormat {
    /// Positive section format (required)
    pub positive_section: FormatSection,
    /// Negative section format (optional)
    pub negative_section: Option<FormatSection>,
    /// Zero value section format (optional)
    pub zero_section: Option<FormatSection>,
    /// Text section format (optional)
    pub text_section: Option<FormatSection>,
}
