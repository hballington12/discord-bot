use crate::coc::get_team;
use crate::{Context, Data, Error};

use ::serenity::all::CreateEmbed;
use poise::serenity_prelude as serenity;

/// Lists all teams in the database
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list_teams(ctx: Context<'_>) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    println!("Database path: {:?}", pool.connect_options());
    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap_or_default()
    );

    // Query all teams from the database using the database function
    let teams = crate::coc::database::get_all_teams(pool).await?;

    if teams.is_empty() {
        ctx.say("No teams found in the database.").await?;
        return Ok(());
    }

    // Format the results
    let response = teams
        .iter()
        .map(|(id, name)| format!("• {} (ID: {})", name, id))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.say(format!("**Teams:**\n{}", response)).await?;
    Ok(())
}

/// Creates a new team in the database
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add_team(
    ctx: Context<'_>,
    #[description = "Name of the team to create"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert team name to lowercase
    let team_name = team_name.to_lowercase();

    println!("checking if team exists");

    // Check if team with this name already exists using the database function
    if let Some(_) = crate::coc::database::get_team_by_name(pool, &team_name).await? {
        ctx.say(format!("Team '{}' already exists!", team_name))
            .await?;
        return Ok(());
    }

    println!("team does not exist");
    println!("creating team");

    // Get the next available team ID
    let next_id = crate::coc::database::get_max_team_id(pool).await? + 1;
    println!("next id: {}", next_id);

    // Insert the team into the database
    crate::coc::database::insert_team(pool, next_id, &team_name).await?;

    // Insert a new set of buildings into the database
    let town_config = &ctx.data().town_config;

    // Get the next available building ID
    let mut building_id = crate::coc::database::get_max_building_id(pool).await? + 1;

    // Loop through the buildings and insert them into the database
    for (building, _) in &town_config.buildings {
        crate::coc::database::insert_team_building(pool, building_id, next_id, building, 1).await?;
        building_id += 1;
    }

    ctx.say(format!(
        "Team '{}' created successfully! (ID: {})",
        team_name, next_id
    ))
    .await?;
    Ok(())
}

/// Deletes a team from the database by name
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn remove_team(
    ctx: Context<'_>,
    #[description = "Name of the team to delete"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert team name to lowercase
    let team_name = team_name.to_lowercase();

    // Check if team with this name exists
    if let None = crate::coc::database::get_team_by_name(pool, &team_name).await? {
        ctx.say(format!("No team found with name '{}'", team_name))
            .await?;
        return Ok(());
    }

    // Delete any buildings associated with this team first
    crate::coc::database::delete_team_buildings(pool, &team_name).await?;

    // Delete the team from the database
    crate::coc::database::delete_team(pool, &team_name).await?;

    ctx.say(format!(
        "Team '{}' has been deleted successfully.",
        team_name,
    ))
    .await?;

    Ok(())
}

/// Adds a player to a team
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add_player(
    ctx: Context<'_>,
    #[description = "Username of the player"] username: String,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert username and team name to lowercase
    let username = username.to_lowercase();
    let team_name = team_name.to_lowercase();

    // Check if the team exists
    let team_id = match crate::coc::database::get_team_by_name(pool, &team_name).await? {
        Some(id) => id,
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Check if the player is already on this team
    if let Some(_) = crate::coc::database::get_team_member(pool, team_id, &username).await? {
        ctx.say(format!(
            "Player '{}' is already a member of team '{}'",
            username, team_name
        ))
        .await?;
        return Ok(());
    }

    // Get the next available member ID
    let next_id = crate::coc::database::get_max_team_member_id(pool).await? + 1;
    println!("next id: {}", next_id);

    // Add the player to the team
    crate::coc::database::insert_team_member(pool, next_id, team_id, &username).await?;

    ctx.say(format!(
        "Successfully added player '{}' to team '{}'",
        username, team_name
    ))
    .await?;

    Ok(())
}

/// Removes a player from all teams
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn remove_player(
    ctx: Context<'_>,
    #[description = "Username of the player to remove"] username: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert username to lowercase
    let username = username.to_lowercase();

    // Check if player exists in any team
    let team_info = crate::coc::database::get_user_team(pool, &username).await?;

    // If player is not in any team, inform the user
    if team_info.is_none() {
        ctx.say(format!("Player '{}' is not a member of any team", username))
            .await?;
        return Ok(());
    }

    // Get team name for feedback message
    let (_, team_name) = team_info.unwrap();

    // Remove the player from all teams
    crate::coc::database::delete_team_members(pool, &username).await?;

    ctx.say(format!(
        "Successfully removed player '{}' from team: {}",
        username, team_name
    ))
    .await?;

    Ok(())
}

/// Creates an embed message with team resources
#[poise::command(slash_command, prefix_command)]
pub async fn create_resource_embed(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let data = ctx.data();
    let pool = &data.database;

    // Get team data from database
    let team_opt = get_team(data, &team_name).await?;

    match team_opt {
        Some(team) => {
            // Use the team's create_message method
            let message_builder = team.create_message().await?;

            // Send the message
            println!("Sending team resource message");
            let msg_result = ctx
                .channel_id()
                .send_message(&ctx.http(), message_builder)
                .await;

            match msg_result {
                Ok(message) => {
                    println!("Team resource message sent successfully");

                    // Record the embed in the team_embeds table
                    let channel_id = ctx.channel_id().get() as i64;
                    let message_id = message.id.get() as i64;
                    let variant = "resources".to_string();

                    // Check if this team already has an embed of this variant
                    let existing = crate::coc::database::get_team_embeds(pool, team.id)
                        .await?
                        .into_iter()
                        .find(|(_, _, _, v)| v == &variant);

                    if let Some((embed_id, _, _, _)) = existing {
                        // Update existing record
                        crate::coc::database::update_team_embed(
                            pool, embed_id, channel_id, message_id,
                        )
                        .await?;
                    } else {
                        // Insert a new embed record
                        crate::coc::database::insert_team_embed(
                            pool, team.id, channel_id, &variant, message_id,
                        )
                        .await?;
                    }

                    ctx.say("Resource embed created and recorded successfully!")
                        .await?;
                }
                Err(why) => {
                    println!("Error sending message: {why:?}");
                    ctx.say(format!("Error sending message: {}", why)).await?;
                }
            }
        }
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
        }
    }

    Ok(())
}

/// Lists all resources for a specific team
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list_team_resources(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();

    // Check if the team exists and get its ID using the database function
    let team_id = match crate::coc::database::get_team_by_name(pool, &team_name).await? {
        Some(id) => id,
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Query resources for this team using the database function
    let resources = crate::coc::database::get_team_resources(pool, team_id).await?;

    if resources.is_empty() {
        ctx.say(format!("No resources found for team '{}'", team_name))
            .await?;
        return Ok(());
    }

    // Format the results
    let response = resources
        .iter()
        .map(|(id, resource_name, quantity)| {
            format!("• **{:?}**: {} (Amount: {})", id, resource_name, quantity)
        })
        .collect::<Vec<_>>()
        .join("\n");

    ctx.say(format!(
        "**Resources for team '{}':**\n{}",
        team_name, response
    ))
    .await?;

    Ok(())
}

/// Updates all registered embeds for a specific team
pub async fn update_team_embeds(
    ctx: &serenity::Context,
    data: &Data,
    team_name: &str,
) -> Result<(usize, Vec<String>), Error> {
    // Get database connection from context data
    let pool = &data.database;

    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();

    // Get team data from database
    let team_opt = get_team(data, &team_name).await?;

    let team = match team_opt {
        Some(team) => team,
        None => return Ok((0, vec!["Team not found".to_string()])), // No team found
    };

    // Find all embeds for this team using the database function
    let records = crate::coc::database::get_team_embeds(pool, team.id).await?;

    if records.is_empty() {
        return Ok((0, vec!["No embeds found for this team".to_string()]));
    }

    let mut updated_count = 0;
    let mut results = Vec::new();

    // Update each embed
    for (embed_id, channel_id_int, message_id_int, variant) in records {
        let channel_id = serenity::ChannelId::new(channel_id_int as u64);
        let message_id = serenity::MessageId::new(message_id_int as u64);

        // Choose embed based on variant
        let embed = match variant.as_str() {
            "buildings" => {
                // For buildings variant, use the buildings embed
                match get_buildings_embed(data, &team_name).await? {
                    Some(buildings_embed) => buildings_embed,
                    None => {
                        results.push(format!(
                            "Failed to generate buildings embed for team {}",
                            team_name
                        ));
                        continue; // Skip to next embed
                    }
                }
            }
            "resources" => {
                // For resources variant, use the resource embed
                team.make_resource_embed()
            }
            unknown => {
                results.push(format!("Unknown embed variant: {}", unknown));
                continue; // Skip to next embed
            }
        };

        // Try to edit the message
        let result = channel_id
            .edit_message(
                &ctx.http,
                message_id,
                serenity::builder::EditMessage::new()
                    .content("This message was edited!")
                    .embed(embed),
            )
            .await;

        match result {
            Ok(_) => {
                updated_count += 1;
                results.push(format!(
                    "Updated {} embed in channel {}",
                    variant, channel_id
                ));
            }
            Err(err) => {
                results.push(format!(
                    "Failed to update {} embed in channel {}: {}",
                    variant, channel_id, err
                ));

                // If message was deleted, remove it from tracking
                if err.to_string().contains("Unknown Message") {
                    crate::coc::database::mark_embed_as_deleted(pool, embed_id).await?;

                    results.push(format!(
                        "Marked message as deleted in database: {} embed in channel {}",
                        variant, channel_id
                    ));
                }
            }
        }
    }

    Ok((updated_count, results))
}

async fn get_buildings_embed(data: &Data, team_name: &str) -> Result<Option<CreateEmbed>, Error> {
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

    // Create the embed
    let mut embed = serenity::builder::CreateEmbed::new()
        .title(format!("🏗️ {} Team Buildings", team_id.1))
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
                .buildings
                .get(&b.building_name.to_lowercase())
                .map(|config| config.name.len())
                .unwrap_or(b.building_name.len())
        })
        .max()
        .unwrap_or(0);

    // Add each building to the table
    for building in &buildings {
        let building_key = building.building_name.to_lowercase();
        let building_config = town_config.buildings.get(&building_key);

        // Get display name and icon
        let (display_name, icon) = match building_config {
            Some(config) => (config.name.clone(), config.icon.clone()),
            None => (building.building_name.clone(), String::new()),
        };

        // Get max level
        let max_level = building_config.map(|c| c.max_level).unwrap_or(9);
        let level_display = if building.level as u32 >= max_level {
            format!("**MAX** ({})", building.level)
        } else {
            format!("{}/{}", building.level, max_level)
        };

        // Format the building entry
        building_list.push_str(&format!(
            "{} `{:<width$}` : Level **{}**\n",
            if icon.is_empty() { "🏢" } else { &icon },
            display_name,
            level_display,
            width = max_name_length
        ));
    }

    // Add the buildings as a field
    embed = embed.field("Buildings", building_list, false);

    Ok(Some(embed))
}

/// Upgrades a building for a team
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn upgrade_building(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
    #[description = "Name of the building to upgrade"] building_name: String,
) -> Result<(), Error> {
    // Get database connection and configs from context data
    let pool = &ctx.data().database;
    let town_config = &ctx.data().town_config;

    // Convert inputs to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();
    let building_name = building_name.to_lowercase();

    // Step 1: Check if the team exists
    let team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    let team_id = match team {
        Some(team) => team.id.ok_or_else(|| Error::from("Team ID is null"))?,
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Step 2: Check if the building exists in the configuration
    if !town_config.buildings.contains_key(&building_name) {
        ctx.say(format!(
            "Building '{}' does not exist in the configuration. Available buildings: {}",
            building_name,
            town_config.get_building_types().join(", ")
        ))
        .await?;
        return Ok(());
    }

    // Step 3: Check if the team has this building and get its current level
    let building = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", level FROM team_buildings 
        WHERE team_id = $1 AND building_name = $2
        "#,
        team_id,
        building_name
    )
    .fetch_optional(pool)
    .await?;

    let (building_id, current_level) = match building {
        Some(building) => (
            building
                .id
                .ok_or_else(|| Error::from("Building ID is null"))?,
            building.level,
        ),
        None => {
            ctx.say(format!(
                "Team '{}' doesn't have a '{}' building. Please check the building name.",
                team_name, building_name
            ))
            .await?;
            return Ok(());
        }
    };

    // Step 4: Check if building is at max level
    let building_config = &town_config.buildings[&building_name];
    if current_level as u32 >= building_config.max_level {
        ctx.say(format!(
            "Building '{}' is already at its maximum level ({})!",
            building_name, current_level
        ))
        .await?;
        return Ok(());
    }

    // Step 6: Get upgrade costs
    let target_level = current_level + 1;
    let costs = town_config.get_upgrade_costs(&building_name, target_level as u32);

    if costs.is_empty() {
        ctx.say(format!(
            "No upgrade costs defined for {} at level {}. Please report this to an admin.",
            building_name, target_level
        ))
        .await?;
        return Ok(());
    }

    // Step 7: Check if the team has enough resources
    let mut missing_resources = Vec::new();
    let mut required_resources = Vec::new();

    for (resource_name, required_amount) in &costs {
        // Get the current amount of this resource
        let resource_query = sqlx::query!(
            r#"
            SELECT quantity FROM resources
            WHERE team_id = $1 AND resource_name = $2
            "#,
            team_id,
            resource_name
        )
        .fetch_optional(pool)
        .await?;

        let current_amount = resource_query.map(|r| r.quantity).unwrap_or(0);

        required_resources.push(format!("`{}`: {}", resource_name, required_amount));

        if current_amount < *required_amount as i64 {
            missing_resources.push(format!(
                "`{}`: have {}/{}",
                resource_name, current_amount, required_amount
            ));
        }
    }

    // Step 8: If missing resources, inform the user and stop
    if !missing_resources.is_empty() {
        ctx.say(format!(
            "Insufficient resources to upgrade {} to level {}!\n\n**Required Resources:**\n{}\n\n**Missing Resources:**\n{}",
            building_name,
            target_level,
            required_resources.join("\n"),
            missing_resources.join("\n")
        ))
        .await?;
        return Ok(());
    }

    // Step 9: Begin transaction to update resources and building level
    let mut tx = pool.begin().await?;

    // Deduct resources
    for (resource_name, amount) in &costs {
        let amount = *amount as i64;
        sqlx::query!(
            r#"
            UPDATE resources
            SET quantity = quantity - $1
            WHERE team_id = $2 AND resource_name = $3
            "#,
            amount,
            team_id,
            resource_name
        )
        .execute(&mut *tx)
        .await?;
    }

    // Upgrade the building
    sqlx::query!(
        r#"
        UPDATE team_buildings
        SET level = level + 1
        WHERE id = $1
        "#,
        building_id
    )
    .execute(&mut *tx)
    .await?;

    // Commit the transaction
    tx.commit().await?;

    // Step 10: Send success message
    let building_display_name = building_config.name.clone();
    let icon = if !building_config.icon.is_empty() {
        format!("{} ", building_config.icon)
    } else {
        String::new()
    };

    ctx.say(format!(
        "{}**{}** upgraded to level **{}** for team **{}**!\n\n**Resources used:**\n{}",
        icon,
        building_display_name,
        target_level,
        team_name,
        required_resources.join("\n")
    ))
    .await?;

    // Step 11: Update any team embeds
    if let Ok((count, _)) =
        update_team_embeds(&ctx.serenity_context(), &ctx.data(), &team_name).await
    {
        if count > 0 {
            ctx.say(format!(
                "Updated {} team embeds with the new information.",
                count
            ))
            .await?;
        }
    }

    Ok(())
}

/// Creates an embed message showing team buildings and their levels
#[poise::command(slash_command, prefix_command)]
pub async fn create_buildings_embed(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let data = ctx.data();
    let pool = &data.database;

    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();

    let embed = match get_buildings_embed(data, &team_name).await? {
        Some(embed) => embed,
        None => {
            ctx.say(format!("No buildings found for team '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Create the message builder with the embed
    let message_builder = serenity::builder::CreateMessage::new().embed(embed);

    // Send the message
    println!("Sending team buildings message");
    let msg_result = ctx
        .channel_id()
        .send_message(&ctx.http(), message_builder)
        .await;

    match msg_result {
        Ok(message) => {
            println!("Team buildings message sent successfully");

            // Record the embed in the team_embeds table
            let channel_id = ctx.channel_id().get() as i64;
            let message_id = message.id.get() as i64;
            let variant = "buildings".to_string();

            // Get the team ID using the database function
            let team_id = match crate::coc::database::get_team_by_name(pool, &team_name).await? {
                Some(id) => id,
                None => {
                    println!("no team found with name '{}'", team_name);
                    return Ok(());
                }
            };

            // Check if this team already has an embed of this variant
            let existing = crate::coc::database::get_team_embeds(pool, team_id)
                .await?
                .into_iter()
                .find(|(_, _, _, v)| v == &variant);

            if let Some((embed_id, _, _, _)) = existing {
                println!("Updating existing record");
                // Update existing record
                crate::coc::database::update_team_embed(pool, embed_id, channel_id, message_id)
                    .await?;
            } else {
                println!("Inserting new record");
                // Insert a new embed record
                crate::coc::database::insert_team_embed(
                    pool, team_id, channel_id, &variant, message_id,
                )
                .await?;
            }

            ctx.say("Buildings embed created and recorded successfully!")
                .await?;
        }
        Err(why) => {
            println!("Error sending message: {why:?}");
            ctx.say(format!("Error sending message: {}", why)).await?;
        }
    }

    Ok(())
}
