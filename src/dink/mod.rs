use std::collections::HashMap;

pub struct ParsedDinkUpdate {
    pub username: String,
    pub loot_string: String,
    pub source: String,
    pub items: HashMap<String, u32>, // Dictionary of item names to quantities
}

pub fn parse_dink_update(content: &str) -> Option<ParsedDinkUpdate> {
    // Extract username (everything before " has looted:")
    let username = if let Some(idx) = content.find(" has looted:") {
        content[..idx].trim().to_string()
    } else {
        println!("Could not find username pattern");
        return None;
    };

    // Extract loot and source
    let mut lines: Vec<&str> = content.lines().collect();
    // Remove empty lines
    lines.retain(|line| !line.trim().is_empty());

    // Find source line (should start with "From: ")
    let source = lines
        .iter()
        .find(|line| line.starts_with("From: "))
        .map(|line| &line[6..]) // Remove "From: " prefix
        .unwrap_or("")
        .to_string();

    // Loot is everything between "has looted:" and "From:" sections
    let mut loot = String::new();
    let mut loot_started = false;
    let mut loot_lines = Vec::new();

    for line in lines {
        if line.starts_with("From: ") {
            break;
        }

        if loot_started {
            if !line.contains("has looted:") {
                loot_lines.push(line);
            }
            if !loot.is_empty() {
                loot.push('\n');
            }
            loot.push_str(line);
        }

        if line.contains("has looted:") {
            loot_started = true;
        }
    }

    // Parse individual items and quantities
    let mut items = HashMap::new();
    for line in loot_lines {
        if let Some((item_name, quantity)) = parse_item_line(line) {
            items.insert(item_name, quantity);
        }
    }

    Some(ParsedDinkUpdate {
        username,
        loot_string: loot.trim().to_string(),
        source,
        items,
    })
}

fn parse_item_line(line: &str) -> Option<(String, u32)> {
    // Pattern: "{quantity} x {item_name} ({some_number})" or similar variations
    if let Some(x_pos) = line.find(" x ") {
        // Extract quantity
        let quantity_str = line[..x_pos].trim();
        let quantity = match quantity_str.parse::<u32>() {
            Ok(num) => num,
            Err(_) => {
                println!("Failed to parse quantity from: {}", line);
                return None;
            }
        };

        // Extract item name - look for either " (" or end of string
        let rest = &line[(x_pos + 3)..]; // Skip " x "
        let item_name = if let Some(paren_pos) = rest.find(" (") {
            rest[..paren_pos].trim()
        } else {
            rest.trim()
        };

        if !item_name.is_empty() {
            return Some((item_name.to_string(), quantity));
        }
    }

    println!("Failed to parse item line: {}", line);
    None
}
