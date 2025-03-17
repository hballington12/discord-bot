use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a single building's configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingConfig {
    pub name: String,
    pub description: String,
    pub starting_level: u32,
    pub max_level: u32,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub upgrade_costs: Vec<Vec<serde_json::Value>>, // [level, resource_name or category, amount]
    #[serde(default)]
    pub benefits: Vec<Vec<serde_json::Value>>, // [level, benefit_description]
}

/// Represents an upgrade cost with either a specific resource or a category
#[derive(Debug, Clone)]
pub enum UpgradeCost {
    Resource(String, u32), // (resource_name, amount)
    Category(String, u32), // (category_name, amount)
}

/// All buildings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TownConfig {
    pub assets: HashMap<String, BuildingConfig>,
    #[serde(default)]
    pub resources: HashMap<String, Vec<String>>,
}

impl TownConfig {
    /// Load building configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the TOML file content
        let content = fs::read_to_string(path)?;

        // Parse the TOML content
        let config: TownConfigRaw = toml::from_str(&content)?;

        // Convert from raw format to our format
        let assets = config.assets;
        let resources = config.resources.unwrap_or_default();

        Ok(TownConfig { assets, resources })
    }

    /// Get all building types
    pub fn get_building_types(&self) -> Vec<String> {
        self.assets.keys().cloned().collect()
    }

    /// Get upgrade costs for a building at a specific level
    pub fn get_upgrade_costs(&self, building_type: &str, level: u32) -> Vec<UpgradeCost> {
        let mut costs = Vec::new();

        if let Some(building) = self.assets.get(building_type) {
            for cost_entry in &building.upgrade_costs {
                if cost_entry.len() >= 3 {
                    // Extract level, resource name, and amount
                    if let (Some(cost_level), Some(resource_or_category), Some(amount)) = (
                        cost_entry.get(0).and_then(|v| v.as_u64()),
                        cost_entry.get(1).and_then(|v| v.as_str()),
                        cost_entry.get(2).and_then(|v| v.as_u64()),
                    ) {
                        if cost_level as u32 == level {
                            // Check if this is a category-based cost
                            if resource_or_category.starts_with("$category:") {
                                let category = resource_or_category
                                    .trim_start_matches("$category:")
                                    .to_string();
                                costs.push(UpgradeCost::Category(category, amount as u32));
                            } else {
                                // Regular resource-based cost
                                costs.push(UpgradeCost::Resource(
                                    resource_or_category.to_string(),
                                    amount as u32,
                                ));
                            }
                        }
                    }
                }
            }
        }

        costs
    }

    /// Get upgrade costs as a HashMap for backward compatibility
    pub fn get_upgrade_costs_map(&self, building_type: &str, level: u32) -> HashMap<String, u32> {
        let mut costs = HashMap::new();

        for cost in self.get_upgrade_costs(building_type, level) {
            match cost {
                UpgradeCost::Resource(resource_name, amount) => {
                    costs.insert(resource_name, amount);
                }
                UpgradeCost::Category(category_name, amount) => {
                    // For HashMap representation, we'll prefix categories
                    costs.insert(format!("$category:{}", category_name), amount);
                }
            }
        }

        costs
    }

    /// Check if a cost is a category-based cost
    pub fn is_category_cost(&self, cost_key: &str) -> bool {
        cost_key.starts_with("$category:")
    }

    /// Extract the category name from a category cost key
    pub fn extract_category_name(&self, cost_key: &str) -> Option<String> {
        if self.is_category_cost(cost_key) {
            Some(cost_key.trim_start_matches("$category:").to_string())
        } else {
            None
        }
    }
}

// Raw structure to parse TOML directly
#[derive(Debug, Deserialize)]
struct TownConfigRaw {
    assets: HashMap<String, BuildingConfig>,
    resources: Option<HashMap<String, Vec<String>>>,
}

/// Initialize the building configuration
pub fn init_assets() -> Result<TownConfig, Box<dyn std::error::Error>> {
    let config_path = "config/asset_list.toml";
    TownConfig::load_from_file(config_path)
}

/// Helper function to format upgrade costs for display
pub fn format_upgrade_costs(costs: &[UpgradeCost]) -> Vec<String> {
    costs
        .iter()
        .map(|cost| match cost {
            UpgradeCost::Resource(name, amount) => format!("`{}`: {}", name, amount),
            UpgradeCost::Category(category, amount) => {
                format!("`{}` (category): {}", category, amount)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_building_config() {
        let result = init_assets();
        assert!(
            result.is_ok(),
            "Failed to load building config: {:?}",
            result.err()
        );

        if let Ok(config) = result {
            // Check that we have town hall
            assert!(config.assets.contains_key("townhall"));

            // Check town hall properties
            if let Some(townhall) = config.assets.get("townhall") {
                assert_eq!(townhall.name, "Town Hall");
                assert_eq!(townhall.max_level, 9);
            }
        }
    }

    #[test]
    fn test_parse_category_costs() {
        let result = init_assets();
        assert!(result.is_ok());

        if let Ok(config) = result {
            // Assuming we have a test building with category-based costs
            let costs = config.get_upgrade_costs("townhall", 4);

            // Check if we can parse category costs
            let has_category_cost = costs.iter().any(|cost| match cost {
                UpgradeCost::Category(_, _) => true,
                _ => false,
            });

            // This test might fail if you haven't added category costs to your TOML yet
            // Just a demonstration of how to test this feature
            println!(
                "Category costs detected in townhall level 4: {}",
                has_category_cost
            );
        }
    }
}
