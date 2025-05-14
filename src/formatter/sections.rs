use crate::types::{ComparisonOperator, FormatSection, NumberFormat};

/// Select the appropriate format section based on the value and format conditions
pub(super) fn select_section(value: f64, format: &NumberFormat) -> &FormatSection {
    // Check for conditional sections first
    if let Some(condition) = &format.positive_section.condition {
        let matches = match condition.operator {
            ComparisonOperator::Eq => value == condition.value,
            ComparisonOperator::Gt => value > condition.value,
            ComparisonOperator::Lt => value < condition.value,
            ComparisonOperator::Ge => value >= condition.value,
            ComparisonOperator::Le => value <= condition.value,
            ComparisonOperator::Ne => value != condition.value,
        };

        if matches {
            return &format.positive_section;
        }
    }

    if let Some(section) = &format.negative_section {
        if let Some(condition) = &section.condition {
            let matches = match condition.operator {
                ComparisonOperator::Eq => value == condition.value,
                ComparisonOperator::Gt => value > condition.value,
                ComparisonOperator::Lt => value < condition.value,
                ComparisonOperator::Ge => value >= condition.value,
                ComparisonOperator::Le => value <= condition.value,
                ComparisonOperator::Ne => value != condition.value,
            };

            if matches {
                return section;
            }
        }
    }

    if let Some(section) = &format.zero_section {
        if let Some(condition) = &section.condition {
            let matches = match condition.operator {
                ComparisonOperator::Eq => value == condition.value,
                ComparisonOperator::Gt => value > condition.value,
                ComparisonOperator::Lt => value < condition.value,
                ComparisonOperator::Ge => value >= condition.value,
                ComparisonOperator::Le => value <= condition.value,
                ComparisonOperator::Ne => value != condition.value,
            };

            if matches {
                return section;
            }
        }
    }

    // If no conditions matched or no conditional sections defined,
    // use standard sign-based selection
    if value < 0.0 {
        if let Some(section) = &format.negative_section {
            if section.condition.is_none() {
                return section;
            }
        }
    } else if value == 0.0 {
        if let Some(section) = &format.zero_section {
            if section.condition.is_none() {
                return section;
            }
        }
    }

    // Default to positive section
    &format.positive_section
}
