use poise::serenity_prelude as serenity;

use crate::coc;
use crate::coc::commands::update_team_embeds;
use crate::coc::database::{
    get_existing_resource, get_team_armory_level, get_user_team, insert_new_resource,
    update_resource_quantity,
};
use crate::{Context, Data, Error};

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
pub async fn handle_message(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &serenity::Message,
) -> Result<(), Error> {
    let embed_count = new_message.embeds.len();
    println!("Received message with {} embed(s)", embed_count);

    for embed in &new_message.embeds {
        println!("loot embed: {:?}", embed);
        let description = match &embed.description {
            Some(desc) => desc,
            None => continue,
        };

        let drop = parse_loot_text(description)?;
        println!("User: {}", drop.user);
        println!("Source: {}", drop.source);
        println!("Items: {:?}", drop.loots);

        println!("processing drop...");
        process_drop(ctx, data, drop).await?;
    }

    Ok(())
}

/// Processes a dink drop
///
/// Check that the user is in the database - if not, return early.
/// For each loot, query the hash table to determine if it is of note.
/// For each noteworthy loot, update the quantity in resources for the player's
/// team.
async fn process_drop(ctx: &serenity::Context, data: &Data, drop: DinkDrop) -> Result<(), Error> {
    let pool = &data.database;
    println!("inside process drop.");

    // Convert username to lowercase for consistent database access
    let username = drop.user.to_lowercase();

    // Check if the user belongs to any team
    let team = match get_user_team(pool, &username).await {
        Ok(Some(team)) => team,
        Ok(None) => {
            println!("User '{}' is not in any team, ignoring drop", drop.user);
            return Ok(());
        }
        Err(e) => {
            println!("Database error when checking user team: {}", e);
            return Err(e.into());
        }
    };

    // try retrieve the combat level of the source
    // return early if no source is found
    let source_level = data.bestiary.get_combat_level(&drop.source);
    if source_level.is_none() {
        println!("No combat level found for source '{}'", drop.source);
        return Ok(());
    }
    let level = source_level.unwrap() as i32;

    // Retrieve the team's armory level and check if they have access to the source's combat level
    let has_access = match get_team_armory_level(pool, level, team.0).await {
        Ok(Some(access)) => access,
        Ok(None) => false,
        Err(e) => {
            println!("Database error when checking team armory level: {}", e);
            return Err(e.into());
        }
    };

    if !has_access {
        println!(
            "Team '{}' doesn't have access to combat level {} monsters",
            team.1,
            source_level.unwrap()
        );
        return Ok(());
    }

    println!(
        "Processing drop for user '{}' of team '{}'",
        drop.user, team.1
    );

    // Process each item in the drop
    for (item_name, quantity) in drop.loots {
        let quantity = quantity as i64;
        let item_name = item_name.to_lowercase();

        println!("querying if item: {} matches pattern...", item_name);

        // Instead, use match pattern function
        let result = coc::patterns::matches_pattern(&item_name, &data.res_patterns.patterns);

        if result {
            println!("Found noteworthy item: {}", item_name);

            // Check if this resource already exists for the team
            let existing_resource = get_existing_resource(pool, team.0, &item_name).await?;

            match existing_resource {
                Some(resource) => {
                    // Update quantity of existing resource
                    let new_quantity = resource + quantity;
                    update_resource_quantity(pool, team.0, &item_name, new_quantity).await?;

                    println!(
                        "Updated resource quantity for team '{}': {} x {} (new total: {})",
                        team.1, item_name, quantity, new_quantity
                    );
                }
                None => {
                    // Insert new resource entry
                    insert_new_resource(pool, team.0, &item_name, quantity).await?;

                    println!(
                        "Added new resource for team '{}': {} x {}",
                        team.1, item_name, quantity
                    );
                }
            }

            // also update any embed resource messages
            let _ = update_team_embeds(ctx, data, &team.1).await;
        }
    }

    Ok(())
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
