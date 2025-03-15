use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Represents a collection of monsters and their combat levels
pub struct Bestiary {
    pub monster_levels: HashMap<String, u32>,
}

impl Bestiary {
    /// Create a new empty bestiary
    pub fn new() -> Self {
        Bestiary {
            monster_levels: HashMap::new(),
        }
    }

    /// Load monster data from the bestiary CSV file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut bestiary = Bestiary::new();

        // Skip the header row
        let mut lines = reader.lines();
        if let Some(Ok(_)) = lines.next() {
            // Process the remaining lines
            for line in lines {
                if let Ok(line) = line {
                    // Split the line by tabs
                    let fields: Vec<&str> = line.split('\t').collect();

                    if fields.len() >= 4 {
                        // Ensure we have at least name and combat level
                        let raw_monster_name = fields[0].trim();

                        // Clean the monster name by removing anything after " - "
                        let clean_monster_name =
                            if let Some(dash_idx) = raw_monster_name.find(" - ") {
                                raw_monster_name[0..dash_idx].trim().to_string()
                            } else {
                                raw_monster_name.to_string()
                            };

                        // Parse the combat level (4th column, index 3)
                        if let Ok(combat_level) = fields[3].trim().parse::<u32>() {
                            // Only insert if the monster doesn't already exist in the bestiary
                            if !bestiary.monster_levels.contains_key(&clean_monster_name) {
                                bestiary
                                    .monster_levels
                                    .insert(clean_monster_name, combat_level);
                            }
                            // else {
                            //     // Optionally, you could log duplicates for debugging
                            //     println!(
                            //         "Skipping duplicate monster entry: {}",
                            //         clean_monster_name
                            //     );
                            // }
                        }
                    }
                }
            }
        }

        Ok(bestiary)
    }

    /// Get the combat level for a monster by name
    pub fn get_combat_level(&self, monster_name: &str) -> Option<u32> {
        self.monster_levels.get(monster_name).copied()
    }

    /// Get all monsters within a specific combat level range
    pub fn get_monsters_in_level_range(
        &self,
        min_level: u32,
        max_level: u32,
    ) -> Vec<(&String, u32)> {
        self.monster_levels
            .iter()
            .filter(|(_, &level)| level >= min_level && level <= max_level)
            .map(|(name, &level)| (name, level))
            .collect()
    }

    /// Get the number of monsters in the bestiary
    pub fn count(&self) -> usize {
        self.monster_levels.len()
    }

    /// Find monsters whose names contain the given search string (case-insensitive)
    pub fn search_monsters(&self, search: &str) -> Vec<(&String, u32)> {
        let search_lower = search.to_lowercase();
        self.monster_levels
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&search_lower))
            .map(|(name, &level)| (name, level))
            .collect()
    }
}

/// Initialize the bestiary from the config file
pub fn init_bestiary() -> Result<Bestiary, io::Error> {
    let bestiary_path = "config/bestiary.csv";
    Bestiary::load_from_file(bestiary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_bestiary() {
        let result = init_bestiary();
        assert!(
            result.is_ok(),
            "Failed to load bestiary: {:?}",
            result.err()
        );

        if let Ok(bestiary) = result {
            // Check that we have monsters loaded
            assert!(bestiary.count() > 0, "No monsters loaded in bestiary");

            // Check a specific monster
            let chicken_level = bestiary.get_combat_level("Chicken");
            assert_eq!(chicken_level, Some(1), "Chicken should be combat level 1");

            let dragon_level = bestiary.get_combat_level("Green dragon");
            assert_eq!(
                dragon_level,
                Some(79),
                "Green dragon should be combat level 79"
            );

            let kurask_level = bestiary.get_combat_level("Kurask");
            assert_eq!(kurask_level, Some(106), "Kurask should be combat level 106");
        }
    }
}
