use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Represents a collection of monsters and their combat levels
pub struct Bestiary {
    pub monster_levels: HashMap<String, u32>,
    pub monster_slayer_levels: HashMap<String, u32>,
}

impl Bestiary {
    /// Create a new empty bestiary
    pub fn new() -> Self {
        Bestiary {
            monster_levels: HashMap::new(),
            monster_slayer_levels: HashMap::new(),
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
                            } else if let Some(paren_idx) = raw_monster_name.find(" (") {
                                raw_monster_name[0..paren_idx].trim().to_string()
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
                        }
                    }
                }
            }
        }

        // Now load slayer data from slayer_list.csv
        bestiary.load_slayer_data()?;

        Ok(bestiary)
    }

    /// Load slayer requirements from the slayer_list.csv file
    fn load_slayer_data(&mut self) -> Result<(), io::Error> {
        let slayer_path = "config/slayer_list.csv";
        let file = File::open(slayer_path)?;
        let reader = BufReader::new(file);

        // Skip the header row
        let mut lines = reader.lines();
        if let Some(Ok(_)) = lines.next() {
            // Process the remaining lines
            for line in lines {
                if let Ok(line) = line {
                    // Split the line by tabs or pipes (depending on the file format)
                    let fields: Vec<&str> = if line.contains('|') {
                        line.split('|').collect()
                    } else {
                        line.split('\t').collect()
                    };

                    if fields.len() >= 3 {
                        // Parse the slayer level requirement (first column)
                        if let Ok(slayer_level) = fields[0].trim().parse::<u32>() {
                            // Only add entries with a slayer level > 1
                            if slayer_level > 1 {
                                // Get the monster name (second column)
                                let monster_name = fields[1].trim().to_string();

                                // Remove the trailing "s" from the monster name
                                let monster_name = if monster_name.ends_with('s') {
                                    monster_name[..monster_name.len() - 1].to_string()
                                } else {
                                    monster_name
                                };

                                // Add the main monster
                                self.monster_slayer_levels
                                    .insert(monster_name.clone(), slayer_level);

                                // Check if there's a Superior variant (7th column)
                                if fields.len() >= 7
                                    && !fields[6].trim().is_empty()
                                    && fields[6].trim() != "N/A"
                                {
                                    let superior_variants = fields[6].trim();

                                    // Process each superior variant (they might be comma separated)
                                    for superior in superior_variants.split(',') {
                                        let superior_name = superior.trim().to_string();
                                        if !superior_name.is_empty() && superior_name != "N/A" {
                                            self.monster_slayer_levels
                                                .insert(superior_name, slayer_level);
                                        }
                                    }
                                }

                                // Check if there are alternative monsters (8th column)
                                if fields.len() >= 8
                                    && !fields[7].trim().is_empty()
                                    && fields[7].trim() != "N/A"
                                {
                                    let alternatives = fields[7].trim();

                                    // Process each alternative (they might be comma separated)
                                    for alternative in alternatives.split(',') {
                                        let alternative_name = alternative.trim().to_string();
                                        if !alternative_name.is_empty() && alternative_name != "N/A"
                                        {
                                            self.monster_slayer_levels
                                                .insert(alternative_name, slayer_level);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the combat level for a monster by name
    pub fn get_combat_level(&self, monster_name: &str) -> Option<u32> {
        self.monster_levels.get(monster_name).copied()
    }

    /// Get the slayer level requirement for a monster by name
    pub fn get_slayer_level(&self, monster_name: &str) -> Option<u32> {
        self.monster_slayer_levels.get(monster_name).copied()
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

    /// Get all monsters with a specific slayer level requirement
    pub fn get_monsters_by_slayer_level(&self, slayer_level: u32) -> Vec<(&String, u32)> {
        self.monster_slayer_levels
            .iter()
            .filter(|(_, &level)| level == slayer_level)
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

            // Check slayer levels
            let kurask_slayer_level = bestiary.get_slayer_level("Kurask");
            assert_eq!(
                kurask_slayer_level,
                Some(70),
                "Kurask should require slayer level 70"
            );

            let king_kurask_slayer_level = bestiary.get_slayer_level("King kurask");
            assert_eq!(
                king_kurask_slayer_level,
                Some(70),
                "King kurask should require slayer level 70"
            );
        }
    }
}
