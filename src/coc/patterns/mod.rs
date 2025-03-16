use regex::Regex;
use serde::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
pub struct ResourcePattern {
    pub pattern: String,
    pub category: String,
}

#[derive(Deserialize)]
pub struct PatternConfig {
    pub resource_pattern: Vec<ResourcePattern>,
}

pub fn load_res_patterns() -> PatternConfig {
    let path = format!(
        "{}/config/resource_list.toml",
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR")
    );
    let config_str = fs::read_to_string(path).expect("Failed to read config");
    toml::from_str(&config_str).expect("Failed to parse TOML")
}

/// Check if input matches any of the patterns
pub fn matches_pattern(input: &str, patterns: &[ResourcePattern]) -> bool {
    patterns
        .iter()
        .any(|p| Regex::new(&p.pattern).unwrap().is_match(input))
}

/// Get the category for a given input string based on pattern matching
pub fn get_resource_category(input: &str, patterns: &[ResourcePattern]) -> String {
    for pattern in patterns {
        if let Ok(regex) = Regex::new(&pattern.pattern) {
            if regex.is_match(input) {
                return pattern.category.clone();
            }
        }
    }

    // Default category if no match
    "miscellaneous".to_string()
}

/// Wrapper function that loads patterns and gets category in one step
pub fn categorize_resource(input: &str) -> String {
    let patterns = load_res_patterns();
    get_resource_category(input, &patterns.resource_pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_categorization() {
        let patterns = vec![
            ResourcePattern {
                pattern: "bones".to_string(),
                category: "bone".to_string(),
            },
            ResourcePattern {
                pattern: ".*ore".to_string(),
                category: "mining".to_string(),
            },
            ResourcePattern {
                pattern: ".oins".to_string(),
                category: "currency".to_string(),
            },
            ResourcePattern {
                pattern: ".*".to_string(),
                category: "miscellaneous".to_string(),
            },
        ];

        assert_eq!(get_resource_category("bones", &patterns), "bone");
        assert_eq!(get_resource_category("iron ore", &patterns), "mining");
        assert_eq!(get_resource_category("gold ore", &patterns), "mining");
        assert_eq!(get_resource_category("coins", &patterns), "currency");
        assert_eq!(get_resource_category("wood", &patterns), "miscellaneous");
    }
}
