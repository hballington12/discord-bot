use poise::serenity_prelude as serenity;
use reqwest::Client;

use crate::coc::commands::update_team_embeds;
use crate::coc::database::{
    get_resource_quantity_by_name, get_team_armory_level, get_team_slayer_level, get_user_team,
    insert_new_resource, update_resource_quantity,
};
use crate::coc::{self, database};
use crate::{Data, Error};

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

pub struct DinkDrop {
    pub user: String,
    pub source: String,
    pub loots: Vec<(String, u32)>,
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
pub async fn process_drop(
    ctx: &serenity::Context,
    data: &Data,
    drop: DinkDrop,
) -> Result<(), Error> {
    let pool = &data.database;

    let username = drop.user.to_lowercase();

    let team = match get_user_team(pool, &username).await {
        Ok(Some(team)) => team,
        Ok(None) => {
            println!("User '{}' is not in any team, ignoring drop", drop.user);
            send_webhook(&drop.user, false, &drop.source, Some("Not in any team")).await?;
            return Ok(());
        }
        Err(e) => {
            println!("Database error when checking user team: {}", e);
            send_webhook(&drop.user, false, &drop.source, Some("Database error")).await?;
            return Err(e.into());
        }
    };

    // Check if the source has a valid combat level
    match data.bestiary.get_combat_level(&drop.source) {
        Some(level) => {
            let source_combat_level = level as i32;

            // Check if team has access to monsters of this combat level
            if !get_team_armory_level(pool, source_combat_level, team.0)
                .await?
                .unwrap_or(false)
            {
                println!(
                    "Team '{}' doesn't have access to combat level {} monsters",
                    team.1, source_combat_level
                );
                send_webhook(
                    &drop.user,
                    false,
                    &drop.source,
                    Some("Team lacks access to this combat level"),
                )
                .await?;
                return Ok(());
            }

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
                        send_webhook(
                            &drop.user,
                            false,
                            &drop.source,
                            Some("Team lacks access to this slayer level"),
                        )
                        .await?;
                        return Ok(());
                    }
                }
                None => {}
            }
        }
        None => {
            let garrisons_level =
                database::get_team_building_level(pool, team.0, "garrisons").await?;

            if drop.source.to_lowercase() == "lunar chest" {
                if garrisons_level < 2 {
                    println!("Team '{}' doesn't have access to Lunar Chests", team.1);
                    send_webhook(
                        &drop.user,
                        false,
                        &drop.source,
                        Some("Team lacks access to Lunar Chests"),
                    )
                    .await?;
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "fortis colosseum" {
                if garrisons_level < 3 {
                    println!("Team '{}' doesn't have access to Fortis Colosseum", team.1);
                    send_webhook(
                        &drop.user,
                        false,
                        &drop.source,
                        Some("Team lacks access to Fortis Colosseum"),
                    )
                    .await?;
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "tombs of amascut"
                || drop.source.to_lowercase() == "tombs of amascut: expert mode"
            {
                if garrisons_level < 4 {
                    println!("Team '{}' doesn't have access to Tombs of Amascut", team.1);
                    send_webhook(
                        &drop.user,
                        false,
                        &drop.source,
                        Some("Team lacks access to Tombs of Amascut"),
                    )
                    .await?;
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "chambers of xeric" {
                if garrisons_level < 5 {
                    println!("Team '{}' doesn't have access to Chambers of Xeric", team.1);
                    send_webhook(
                        &drop.user,
                        false,
                        &drop.source,
                        Some("Team lacks access to Chambers of Xeric"),
                    )
                    .await?;
                    return Ok(());
                }
            } else if drop.source.to_lowercase() == "theatre of blood" {
                if garrisons_level < 6 {
                    println!("Team '{}' doesn't have access to Theatre of Blood", team.1);
                    send_webhook(
                        &drop.user,
                        false,
                        &drop.source,
                        Some("Team lacks access to Theatre of Blood"),
                    )
                    .await?;
                    return Ok(());
                }
            } else {
                println!("No combat level found for source '{}'", drop.source);
                send_webhook(&drop.user, false, &drop.source, Some("Invalid source")).await?;
                return Ok(());
            }
        }
    };

    // Process each item in the drop
    for (item_name, quantity) in drop.loots {
        let quantity = quantity as i64;
        let item_name = item_name.to_lowercase();

        let result =
            coc::patterns::matches_pattern(&item_name, &data.res_patterns.resource_pattern);

        if !result {
            continue;
        }

        let category =
            coc::patterns::get_resource_category(&item_name, &data.res_patterns.resource_pattern);

        let quantity =
            coc::database::calculate_resource_total(pool, quantity as i32, team.0, &category)
                .await?;

        let existing_resource = get_resource_quantity_by_name(pool, team.0, &item_name).await?;

        match existing_resource {
            Some(resource) => {
                let new_quantity = resource + quantity as i64;
                update_resource_quantity(pool, team.0, &item_name, new_quantity).await?;
            }
            None => {
                insert_new_resource(pool, team.0, &item_name, &category, quantity as i64).await?;
            }
        }
    }

    // Throttle embed updates - only update after certain time interval
    let team_name = &team.1;
    let update_needed = should_update_team_embeds(data, team_name).await;

    if update_needed {
        println!("Updating team embeds for '{}'", team_name);
        update_team_embeds(ctx, data, team_name).await?;
    } else {
        println!("Skipping team embed update (throttled) for '{}'", team_name);
    }

    send_webhook(
        &drop.user,
        true,
        &drop.source,
        Some("Drop processed successfully"),
    )
    .await?;

    Ok(())
}

// Add to Data struct a field:
// last_embed_update: Arc<tokio::sync::Mutex<HashMap<String, Instant>>>

async fn should_update_team_embeds(data: &Data, team_name: &str) -> bool {
    let mut updates = data.last_embed_update.lock().await;
    let now = std::time::Instant::now();

    // Only update embeds once every 30 seconds per team
    const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

    if let Some(last_update) = updates.get(team_name) {
        if now.duration_since(*last_update) < UPDATE_INTERVAL {
            return false;
        }
    }

    // Update the timestamp for this team
    updates.insert(team_name.to_string(), now);
    // println!("Updated last embed update time for team '{}'", team_name);
    true
}

/// Sends a Discord webhook message.
///
/// # Arguments
/// * `player_name` - The name of the player.
/// * `status` - A boolean value indicating some condition (e.g., success or failure).
/// * `source` - The source of the drop.
/// * `optional_message` - An optional string to include in the webhook message.
pub async fn send_webhook(
    player_name: &str,
    status: bool,
    source: &str,
    optional_message: Option<&str>,
) -> Result<(), Error> {
    let webhook_url = match std::env::var("DISCORD_WEBHOOK_URL") {
        Ok(url) => url,
        Err(e) => {
            eprintln!(
                "❌ send_webhook: DISCORD_WEBHOOK_URL environment variable not set: {}",
                e
            );
            return Err("DISCORD_WEBHOOK_URL environment variable not set".into());
        }
    };

    let status_emoji = if status { "✅" } else { "❌" };
    let message = if status {
        format!("{} {} {}", player_name, source, status_emoji)
    } else {
        format!(
            "{} {} {} {}",
            player_name,
            source,
            status_emoji,
            optional_message.unwrap_or("")
        )
    };

    let client = Client::new();
    let payload = serde_json::json!({
        "content": message
    });

    let _response = client.post(&webhook_url).json(&payload).send().await;

    // match response {
    //     Ok(resp) => {
    //         if resp.status().is_success() {
    //         } else if resp.status().as_u16() == 429 {
    //             // Handle rate limit exceeded
    //             if let Some(retry_after) = resp.headers().get("Retry-After") {
    //                 let retry_str = retry_after.to_str().unwrap_or("unknown");
    //                 println!(
    //                     "⚠️ send_webhook: Rate limit exceeded. Retry after: {} seconds",
    //                     retry_str
    //                 );
    //             } else {
    //                 println!(
    //                     "⚠️ send_webhook: Rate limit exceeded but no Retry-After header found"
    //                 );
    //             }

    //             println!(
    //                 "⚠️ send_webhook: Full response headers: {:?}",
    //                 resp.headers()
    //             );
    //         } else {
    //             eprintln!(
    //                 "❌ send_webhook: Failed to send webhook: Status {}",
    //                 resp.status()
    //             );
    //             println!("❌ send_webhook: Response headers: {:?}", resp.headers());

    //             // Try to get response body for more details
    //             match resp.text().await {
    //                 Ok(body) => println!("❌ send_webhook: Response body: {}", body),
    //                 Err(e) => println!("❌ send_webhook: Couldn't read response body: {}", e),
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("❌ send_webhook: Failed to send webhook: {}", e);
    //         println!("❌ send_webhook: Error details: {:?}", e);
    //         return Err(e.into());
    //     }
    // }

    Ok(())
}

/// Parse loot text into structured format
///
/// Input format: "User has looted: \n\n# x [Item](url) (value)\n# x [Item](url) (value)\nFrom: [Source](url)"
/// Output: (username, Vec<(item_name, quantity)>, source)
pub fn parse_loot_text(text: &str) -> Result<DinkDrop, Error> {
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
