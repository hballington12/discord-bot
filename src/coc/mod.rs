use std::collections::{HashMap, HashSet};

pub mod commands;

// Add this struct to track team information
#[derive(Clone)]
pub struct TeamInfo {
    pub id: u32,
    pub name: String,
}

// Create a function to initialize the set of interesting items
pub fn setup_interesting_items() -> HashSet<String> {
    let mut items = HashSet::new();

    // Add items that we're interested in
    items.insert("Coal".to_string());
    items.insert("Bones".to_string());
    items.insert("Water rune".to_string());
    items.insert("Nature rune".to_string());
    items.insert("Death rune".to_string());
    items.insert("Blood rune".to_string());
    items.insert("Law rune".to_string());
    items.insert("Soul rune".to_string());

    // Add more items as needed

    items
}

// Create a function to initialize the username to team mapping
pub fn setup_username_teams() -> HashMap<String, TeamInfo> {
    let mut username_map = HashMap::new();

    // Team 1
    let team1 = TeamInfo {
        id: 1,
        name: "Dragon Hunters".to_string(),
    };

    username_map.insert("Solo H".to_string(), team1.clone());
    username_map.insert("Group Chrog".to_string(), team1.clone());
    username_map.insert("Magic Pro".to_string(), team1);

    // Team 2
    let team2 = TeamInfo {
        id: 2,
        name: "Goblin Squad".to_string(),
    };

    username_map.insert("Goblin_1".to_string(), team2.clone());
    username_map.insert("Goblin_2".to_string(), team2.clone());
    username_map.insert("Goblin_King".to_string(), team2);

    // Add more teams and usernames as needed

    username_map
}
