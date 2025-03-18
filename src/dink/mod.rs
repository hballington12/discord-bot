use poise::serenity_prelude as serenity;

use crate::coc::commands::update_team_embeds;
use crate::coc::database::{
    get_resource_quantity_by_name, get_team_armory_level, get_team_slayer_level, get_user_team,
    insert_new_resource, update_resource_quantity,
};
use crate::coc::{self, database};
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
    // let embed_count = new_message.embeds.len();
    // println!("Received message with {} embed(s)", embed_count);

    for embed in &new_message.embeds {
        // println!("loot embed: {:?}", embed);
        let description = match &embed.description {
            Some(desc) => desc,
            None => continue,
        };

        // println!("dink embed description is: '{}'", description);

        let drop = parse_loot_text(description)?;

        println!(
            "Processing drop: User: {}, Source: {}, Items: {:?}",
            drop.user, drop.source, drop.loots
        );

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

    // Check if the source has a valid combat level
    match data.bestiary.get_combat_level(&drop.source) {
        Some(level) => {
            let source_combat_level = level as i32;

            //// TEMPORARY DISABLE
            // // Check if team has access to monsters of this combat level
            // if !get_team_armory_level(pool, source_combat_level, team.0)
            //     .await?
            //     .unwrap_or(false)
            // {
            //     println!(
            //         "Team '{}' doesn't have access to combat level {} monsters",
            //         team.1, source_combat_level
            //     );
            //     return Ok(());
            // }

            match data.bestiary.get_slayer_level(&drop.source) {
                Some(level) => {
                    println!("Slayer level for source '{}': {}", drop.source, level);

                    // Check if team has necessary slayer level
                    if !get_team_slayer_level(pool, level as i32, team.0)
                        .await?
                        .unwrap_or(false)
                    {
                        println!(
                            "Team '{}' doesn't have access to slayer level {} monsters",
                            team.1, level
                        );
                        return Ok(());
                    }
                }
                None => {}
            }
        }
        None => {
            // retrieve the teams garrisons level
            let garrisons_level =
                database::get_team_building_level(pool, team.0, "garrisons").await?;

            // allow the code to proceed if the source is a raid drop
            // valid raids are:
            // "Tombs of Amascut" garrisons level 2
            // "Chambers of Xeric", garrisons level 3
            // "Theatre of Blood", garrisons level 4

            // check the different patterns and return early if garrisons level is not met
            // first check toms of amascut
            if drop.source.to_lowercase() == "tombs of amascut" {
                if garrisons_level < 2 {
                    println!("Team '{}' doesn't have access to Tombs of Amascut", team.1);
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "chambers of xeric" {
                if garrisons_level < 3 {
                    println!("Team '{}' doesn't have access to Chambers of Xeric", team.1);
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "theatre of blood" {
                if garrisons_level < 4 {
                    println!("Team '{}' doesn't have access to Theatre of Blood", team.1);
                    return Ok(());
                }
            } else {
                println!("No combat level found for source '{}'", drop.source);
                return Ok(());
            }
        }
    };

    // Process each item in the drop
    for (item_name, quantity) in drop.loots {
        let quantity = quantity as i64;
        let item_name = item_name.to_lowercase();

        // Check if item matches resource pattern
        let result =
            coc::patterns::matches_pattern(&item_name, &data.res_patterns.resource_pattern);

        // Skip this item if it doesn't match any resource pattern
        if !result {
            continue;
        }

        // get the category for the item
        let category =
            coc::patterns::get_resource_category(&item_name, &data.res_patterns.resource_pattern);

        println!("Item match found with category: {}", category);
        // get the modified resource amount
        let quantity =
            coc::database::calculate_resource_total(pool, quantity as i32, team.0, &category)
                .await?;

        // Check if this resource already exists for the team
        let existing_resource = get_resource_quantity_by_name(pool, team.0, &item_name).await?;
        println!("Existing resource: {:?}", existing_resource);

        match existing_resource {
            Some(resource) => {
                // Update quantity of existing resource
                let new_quantity = resource + quantity as i64;
                update_resource_quantity(pool, team.0, &item_name, new_quantity).await?;

                println!(
                    "Updated resource quantity for team '{}': {} x {} (new total: {})",
                    team.1, item_name, quantity, new_quantity
                );
            }
            None => {
                // Insert new resource entry
                insert_new_resource(pool, team.0, &item_name, &category, quantity as i64).await?;

                println!(
                    "Added new resource for team '{}': {} x {}",
                    team.1, item_name, quantity
                );
            }
        }
    }

    // also update any embed resource messages
    let _ = update_team_embeds(ctx, data, &team.1).await?;

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
