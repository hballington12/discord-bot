use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a single building's configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingConfig {
    pub name: String,
    pub description: String,
    #[serde(default = "default_starting_level")]
    pub starting_level: u32,
    pub max_level: u32,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub upgrade_costs: Vec<Vec<serde_json::Value>>, // [level, resource_name, amount]
    #[serde(default)]
    pub benefits: Vec<Vec<serde_json::Value>>, // [level, benefit_description]
}

/// All buildings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TownConfig {
    pub buildings: HashMap<String, BuildingConfig>,
    #[serde(default)]
    pub resources: HashMap<String, Vec<String>>,
}

// Helper function for default starting level
fn default_starting_level() -> u32 {
    1
}

impl TownConfig {
    /// Load building configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the TOML file content
        let content = fs::read_to_string(path)?;

        // Parse the TOML content
        let config: TownConfigRaw = toml::from_str(&content)?;

        // Convert from raw format to our format
        let buildings = config.buildings;
        let resources = config.resources.unwrap_or_default();

        Ok(TownConfig {
            buildings,
            resources,
        })
    }

    /// Get all building types
    pub fn get_building_types(&self) -> Vec<String> {
        self.buildings.keys().cloned().collect()
    }

    /// Get upgrade costs for a building at a specific level
    pub fn get_upgrade_costs(&self, building_type: &str, level: u32) -> HashMap<String, u32> {
        let mut costs = HashMap::new();

        if let Some(building) = self.buildings.get(building_type) {
            for cost_entry in &building.upgrade_costs {
                if cost_entry.len() >= 3 {
                    // Extract level, resource name, and amount
                    if let (Some(cost_level), Some(resource), Some(amount)) = (
                        cost_entry.get(0).and_then(|v| v.as_u64()),
                        cost_entry.get(1).and_then(|v| v.as_str()),
                        cost_entry.get(2).and_then(|v| v.as_u64()),
                    ) {
                        if cost_level as u32 == level {
                            costs.insert(resource.to_string(), amount as u32);
                        }
                    }
                }
            }
        }

        costs
    }
}

// Raw structure to parse TOML directly
#[derive(Debug, Deserialize)]
struct TownConfigRaw {
    buildings: HashMap<String, BuildingConfig>,
    resources: Option<HashMap<String, Vec<String>>>,
}

/// Initialize the building configuration
pub fn init_buildings() -> Result<TownConfig, Box<dyn std::error::Error>> {
    let config_path = "config/building_list.toml";
    TownConfig::load_from_file(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_building_config() {
        let result = init_buildings();
        assert!(
            result.is_ok(),
            "Failed to load building config: {:?}",
            result.err()
        );

        if let Ok(config) = result {
            // Check that we have town hall
            assert!(config.buildings.contains_key("townhall"));

            // Check town hall properties
            if let Some(townhall) = config.buildings.get("townhall") {
                assert_eq!(townhall.name, "Town Hall");
                assert_eq!(townhall.max_level, 9);
            }
        }
    }
}
