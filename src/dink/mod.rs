use poise::serenity_prelude as serenity;

use crate::{dink, Error};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_loot_text() {
        let input = "Solo H has looted: \n\n1 x [Bones](https://oldschool.runescape.wiki/w/Special:Search?search=Bones) (62)\n15 x [Coins](https://oldschool.runescape.wiki/w/Special:Search?search=Coins) (15)\nFrom: [Man](https://oldschool.runescape.wiki/w/Special:Search?search=Man)";

        let dink_drop = parse_loot_text(input).unwrap();

        assert_eq!(dink_drop.user, "Solo H");
        assert_eq!(
            dink_drop.loots,
            vec![("Bones".to_string(), 1), ("Coins".to_string(), 15)]
        );
        assert_eq!(dink_drop.source, "Man");
    }
}

struct DinkDrop {
    user: String,
    source: String,
    loots: Vec<(String, u32)>,
}

impl DinkDrop {
    pub fn new(user: String, source: String, loots: Vec<(String, u32)>) -> Self {
        Self {
            user,
            source,
            loots,
        }
    }
}
/// Handles a message sent in the dink channel.
/// If the message contains embeds, attempts to parse each embed description
/// into a `DinkDrop` struct for processing.
pub fn handle_message(new_message: &serenity::Message) {
    let embed_count = new_message.embeds.len();
    println!("Received message with {} embed(s)", embed_count);

    for embed in &new_message.embeds {
        let description = match &embed.description {
            Some(desc) => desc,
            None => continue,
        };

        match parse_loot_text(description) {
            Ok(drop) => {
                println!("User: {}", drop.user);
                println!("Source: {}", drop.source);
                println!("Items: {:?}", drop.loots);

                //
            }
            Err(e) => println!("Failed to parse embed: {}", e),
        }
    }
}

/// Parse loot text into structured format
///
/// Input format: "User has looted: \n\n# x [Item](url) (value)\n# x [Item](url) (value)\nFrom: [Source](url)"
/// Output: (username, Vec<(item_name, quantity)>, source)
fn parse_loot_text(text: &str) -> Result<DinkDrop, Error> {
    // Extract username (assumes format "Username has looted:")
    let username = match text.split(" has looted:").next() {
        Some(name) => name.trim().to_string(),
        None => return Err("Could not find username in loot text".into()),
    };

    // Return err if username longer than 15 characters
    if username.len() > 15 {
        return Err("Username is too long (exceeds 15 characters)".into());
    }

    // Split the text into lines and process each line%USERNAME% has looted:
    let mut loots = Vec::new();
    let mut source = String::new();

    for line in text.lines() {
        // Parse loot lines (format: "# x [Item](url) (value)")
        if let Some(captures) = line.trim().split_once(" x [") {
            let quantity = captures.0.trim().parse::<u32>().unwrap_or(0);
            if let Some(item_name) = captures.1.split("](").next() {
                loots.push((item_name.to_string(), quantity));
            }
        }

        // Parse source line (format: "From: [Source](url)")
        if line.trim().starts_with("From: [") {
            if let Some(src) = line.trim()[7..].split("](").next() {
                source = src.to_string();
            }
        }
    }

    if loots.is_empty() {
        return Err("No loots found in loot text".into());
    }

    Ok(DinkDrop::new(username, source, loots))
}
