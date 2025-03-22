use crate::{Data, Error};

use ::serenity::all::CreateEmbed;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

pub async fn get_buildings_embed(
    data: &Data,
    team_name: &str,
) -> Result<Option<CreateEmbed>, Error> {
    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();
    let town_config = &data.town_config;
    let pool = &data.database;

    // Check if the team exists and get its ID
    let team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", name FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    // If the team doesn't exist, inform the user and return early
    let team_id = match team {
        Some(team) => (
            team.id.ok_or_else(|| Error::from("Team ID is null"))?,
            team.name,
        ),
        None => {
            println!("no team found with name '{}'", team_name);
            return Ok(None);
        }
    };

    // Query buildings for this team
    let buildings = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", building_name, level
        FROM team_buildings
        WHERE team_id = $1
        ORDER BY building_name ASC
        "#,
        team_id.0
    )
    .fetch_all(pool)
    .await?;

    if buildings.is_empty() {
        println!("no buildings found for team '{}'", team_id.1);
        return Ok(None);
    }

    // Query resource multipliers for this team
    let multipliers = sqlx::query!(
        r#"
        SELECT 
            tb.building_name,
            rm.resource_category,
            rm.multiplier,
            rm.flat_bonus
        FROM 
            team_buildings tb
        JOIN 
            resource_multiplier_mapping rm 
            ON tb.building_name = rm.building_name AND tb.level = rm.building_level
        WHERE 
            tb.team_id = $1
        "#,
        team_id.0
    )
    .fetch_all(pool)
    .await?;

    // Create a mapping of building name to (resource category, multiplier, flat_bonus)
    let mut building_bonuses: HashMap<String, Vec<(String, f64, i64)>> = HashMap::new();

    for m in multipliers {
        // Skip entries with default multiplier (1.0) and no bonus (0)
        if m.multiplier == 1.0 && m.flat_bonus == 0 {
            continue;
        }

        let entry = building_bonuses
            .entry(m.building_name.to_lowercase())
            .or_insert_with(Vec::new);

        entry.push((m.resource_category, m.multiplier, m.flat_bonus));
    }

    // Create a mapping of special building attributes
    let mut building_special_info: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Find the armory and query its combat level access
    let armory = buildings
        .iter()
        .find(|b| b.building_name.to_lowercase() == "armory");
    if let Some(armory) = armory {
        let combat_access = sqlx::query!(
            r#"
            SELECT max_combat_level
            FROM armory_combat_mapping
            WHERE armory_level = $1
            "#,
            armory.level
        )
        .fetch_optional(pool)
        .await?;

        if let Some(combat_data) = combat_access {
            let mut armory_info = HashMap::new();
            armory_info.insert(
                "combat_level".to_string(),
                format!("{}", combat_data.max_combat_level),
            );
            building_special_info.insert("armory".to_string(), armory_info);
        }
    }

    // Find the slayer_master and query its slayer_level access
    let slayer_master = buildings
        .iter()
        .find(|b| b.building_name.to_lowercase() == "slayer_master");
    if let Some(slayer_master) = slayer_master {
        let slayer_access = sqlx::query!(
            r#"
            SELECT slayer_level
            FROM slayer_master_level_mapping
            WHERE slayer_master_level = $1
            "#,
            slayer_master.level
        )
        .fetch_optional(pool)
        .await?;

        if let Some(slayer_data) = slayer_access {
            let mut slayer_info = HashMap::new();
            slayer_info.insert(
                "slayer_level".to_string(),
                format!("{}", slayer_data.slayer_level),
            );
            building_special_info.insert("slayer_master".to_string(), slayer_info);
        }
    }

    // Create the embed
    let mut embed = serenity::builder::CreateEmbed::new()
        .title(format!("🏗️ Buildings for: {}", team_id.1))
        .description(format!(
            "Building infrastructure for team **{}**",
            team_id.1
        ))
        .footer(serenity::builder::CreateEmbedFooter::new(format!(
            "Team ID: {:?}",
            team_id.0
        )))
        .timestamp(serenity::model::Timestamp::now());

    // Group buildings by type
    let mut building_list = String::new();

    // Find the longest building name for padding
    let max_name_length = buildings
        .iter()
        .map(|b| {
            town_config
                .assets
                .get(&b.building_name.to_lowercase())
                .map(|config| config.name.len())
                .unwrap_or(b.building_name.len())
        })
        .max()
        .unwrap_or(0);

    // Split the buildings into Town Hall and others
    let mut town_hall_entry = String::new();
    let mut other_buildings = Vec::new();

    // First pass: find and format the Town Hall entry
    for building in &buildings {
        let building_key = building.building_name.to_lowercase();

        // Check if this is the Town Hall
        if building_key == "townhall" || building_key == "town_hall" {
            let building_config = town_config.assets.get(&building_key);

            // Get display name and icon
            let (display_name, icon) = match building_config {
                Some(config) => (config.name.clone(), config.icon.clone()),
                None => (building.building_name.clone(), String::new()),
            };

            // Format the level display
            let max_level = building_config.map(|c| c.max_level).unwrap_or(9);
            let level_display = if building.level as u32 >= max_level {
                format!("**MAX** ({})", building.level)
            } else {
                format!("{}/{}", building.level, max_level)
            };

            // Format the Town Hall entry
            town_hall_entry = format!(
                "{} `{:<width$}` : Level **{}**\n",
                if icon.is_empty() { "🏢" } else { &icon },
                display_name,
                level_display,
                width = max_name_length
            );

            // Don't add to other_buildings
        } else {
            other_buildings.push(building);
        }
    }

    // Add Town Hall at the top if found
    if !town_hall_entry.is_empty() {
        building_list.push_str(&town_hall_entry);
        building_list.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"); // Add a separator
    }

    // Add all other buildings
    for building in other_buildings {
        let building_key = building.building_name.to_lowercase();
        let building_config = town_config.assets.get(&building_key);

        // Get display name and icon
        let (display_name, icon) = match building_config {
            Some(config) => (config.name.clone(), config.icon.clone()),
            None => (building.building_name.clone(), String::new()),
        };

        // Format the level display - no level 0 case anymore
        let max_level = building_config.map(|c| c.max_level).unwrap_or(9);
        let level_display = if building.level as u32 >= max_level {
            format!("**MAX** ({})", building.level)
        } else {
            format!("{}/{}", building.level, max_level)
        };

        // Format the basic building info
        let mut building_entry = format!(
            "{} `{:<width$}` : Level **{}**",
            if icon.is_empty() { "🏢" } else { &icon },
            display_name,
            level_display,
            width = max_name_length
        );

        // Add special info for certain buildings
        // Special handling for Armory (Combat Level Access)
        if building_key == "armory" {
            if let Some(armory_info) = building_special_info.get("armory") {
                if let Some(combat_level) = armory_info.get("combat_level") {
                    building_entry.push_str(&format!(
                        "\n   ┗ **NPC Combat Level**: Up to level {}\n",
                        combat_level
                    ));
                }
            }
        }

        // Add special info for certain buildings
        // Special handling for Slayer Master (Combat Level Access)
        if building_key == "slayer_master" {
            if let Some(slayer_info) = building_special_info.get("slayer_master") {
                if let Some(combat_level) = slayer_info.get("slayer_level") {
                    building_entry.push_str(&format!(
                        "\n   ┗ **NPC Slayer Level**: Up to level {}\n",
                        combat_level
                    ));
                }
            }
        }

        // Special handling for Garrisons (Raid Access)
        if building_key == "garrisons" {
            let raid_access = match building.level {
                2 => "Access to: Colosseum",
                3 => "Access to: Colosseum, ToA",
                4 => "Access to: Colosseum, ToA and CoX",
                5 => "Access to: Colosseum + all raids",
                _ => "No special content access",
            };

            building_entry.push_str(&format!("\n   ┗ **Raid Access**: {}\n", raid_access));
        }

        // Add bonus information if this building provides any
        if let Some(bonuses) = building_bonuses.get(&building_key) {
            building_entry.push_str("\n");

            // Show resource bonuses in a nice format
            for (resource_category, multiplier, flat_bonus) in bonuses {
                let mut bonus_text = String::new();

                // Add multiplier if not default (1.0)
                if *multiplier > 1.0 {
                    if !bonus_text.is_empty() {
                        bonus_text.push_str(", ");
                    }
                    bonus_text.push_str(&format!("{:.1}x multiplier", multiplier));
                }

                // Add flat bonus if present
                if *flat_bonus > 0 {
                    bonus_text.push_str(&format!(", +{} bonus", flat_bonus));
                }

                // Add the formatted resource bonus line
                building_entry.push_str(&format!(
                    "   ┗ {}: {}\n",
                    resource_category_display(resource_category),
                    bonus_text
                ));
            }
        }

        // Add the entry to our list
        building_list.push_str(&building_entry);

        // Add a newline after each entry
        building_list.push_str("\n");
    }

    // Add the buildings as a field
    embed = embed.field("Buildings", building_list, false);

    Ok(Some(embed))
}

// Helper function to make resource categories more presentable
fn resource_category_display(category: &str) -> &str {
    match category {
        "mining" => "Mining Resources",
        "wood" => "Wood Resources",
        "fishing" => "Fishing Resources",
        "herb" => "Herblore Resources",
        "farming" => "Farming Resources",
        "currency" => "Coin Rewards",
        "rune" => "Rune Yields",
        "crafting" => "Crafting Materials",
        "hunting" => "Hunting Yields",
        _ => category,
    }
}

// ...existing code...

pub async fn get_teams_townhall_levels(data: &Data) -> Result<Option<CreateEmbed>, Error> {
    let pool = &data.database;
    let town_config = &data.town_config;

    // Query all teams and their town hall levels
    let teams_data = sqlx::query!(
        r#"
        SELECT 
            t.id as "team_id: i32", 
            t.name as team_name, 
            tb.level as town_hall_level
        FROM 
            teams t
        LEFT JOIN 
            team_buildings tb ON t.id = tb.team_id
        WHERE 
            LOWER(tb.building_name) = 'townhall' 
            OR LOWER(tb.building_name) = 'town_hall'
        ORDER BY 
            tb.level DESC, 
            t.name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    // Get the maximum town hall level from config for reference
    let max_townhall_level = town_config
        .assets
        .get("townhall")
        .or_else(|| town_config.assets.get("town_hall"))
        .map(|config| config.max_level)
        .unwrap_or(10); // Default to 10 if not found

    // Create the embed
    let mut embed = serenity::builder::CreateEmbed::new()
        .title("🏰 Team Town Hall Levels")
        .description(format!(
            "Overview of all teams' town hall progression. Maximum level: **{}**",
            max_townhall_level
        ))
        .footer(serenity::builder::CreateEmbedFooter::new(format!(
            "Total teams: {} • Updated: {}",
            teams_data.len(),
            chrono::Local::now().format("%d %b %Y %H:%M")
        )))
        .color(0x3498db) // Nice blue color
        .timestamp(serenity::model::Timestamp::now());

    // Create a nicely formatted table for the teams
    let mut table = "```\n".to_string();
    table.push_str("📊 Rank | 🏷️ Team Name      | 🏰 Level | 📈 Progress\n");
    table.push_str("--------|-------------------|----------|------------\n");

    // Add teams to the table
    for (index, team) in teams_data.iter().enumerate() {
        let rank = index + 1;
        let team_name = &team.team_name;
        let th_level = team.town_hall_level.expect("Town Hall level is null");

        // Calculate progress percentage and create progress bar
        let percentage = (th_level as f64 / max_townhall_level as f64 * 100.0).min(100.0);
        let progress_bar = if th_level as u32 >= max_townhall_level {
            "🌟 MAXED".to_string()
        } else {
            let filled = (percentage / 10.0).round() as usize;
            let unfilled = 10 - filled;
            format!("{}{}", "█".repeat(filled), "░".repeat(unfilled))
        };

        // Format the rank with trophy/medal emoji for top 3
        let rank_display = match rank {
            1 => "#1🥇",
            2 => "#2🥈",
            3 => "#3🥉",
            _ => &format!("#{}     ", rank),
        };

        // Format name (truncate if too long)
        let name_display = if team_name.len() > 17 {
            format!("{}...", &team_name[0..14])
        } else {
            team_name.clone()
        };

        // Add the row to the table
        table.push_str(&format!(
            "{:<6} | {:<17} | {:<8} | {}\n",
            rank_display,
            name_display,
            format!("{}/{}", th_level, max_townhall_level),
            progress_bar
        ));
    }

    table.push_str("```");

    // Handle case with no teams
    if teams_data.is_empty() {
        table = "```\nNo teams with town halls found.\n```".to_string();
    }

    // Add the table to the embed
    embed = embed.field("Town Hall Rankings", table, false);

    // Add some helpful context in a separate field
    embed = embed.field(
        "💡 About Town Halls", 
        "Town Hall level determines the maximum level of other buildings and unlocks new construction options.\n\
        Upgrading your Town Hall should be a team priority to access higher-tier content and bonuses.",
        false
    );

    Ok(Some(embed))
}
// ...existing code...
