use regex::Regex;
use serde::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
pub struct PatternConfig {
    pub patterns: Vec<String>,
}

pub fn load_res_patterns() -> PatternConfig {
    let path = format!(
        "{}/config/resource_list.toml",
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR")
    );
    let config_str = fs::read_to_string(path).expect("Failed to read config");
    toml::from_str(&config_str).expect("Failed to parse TOML")
}

pub fn matches_pattern(input: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|p| Regex::new(p).unwrap().is_match(input))
}
