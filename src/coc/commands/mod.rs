use crate::{
    coc::{
        self,
        database::{get_resource_quantity_by_name, insert_new_resource, update_resource_quantity},
        get_team,
    },
    Context, Data, Error,
};

use poise::serenity_prelude as serenity;

mod embed;
pub mod helper;

/// Lists all teams in the database
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
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
        ctx.send(
            poise::CreateReply::default()
                .content("No teams found in the database.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Format the results
    let response = teams
        .iter()
        .map(|(id, name)| format!("• {} (ID: {})", name, id))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(
        poise::CreateReply::default()
            .content(format!("**Teams:**\n{}", response))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Creates a new team in the database
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
pub async fn add_team(
    ctx: Context<'_>,
    #[description = "Name of the team to create"] team_name: String,
    #[description = "Handicap value for the team (default: 1)"] handicap: Option<i32>,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert team name to lowercase
    let team_name = team_name.to_lowercase();

    println!("checking if team exists");

    // Check if team with this name already exists using the database function
    if let Some(_) = crate::coc::database::get_team_by_name(pool, &team_name).await? {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("Team '{}' already exists!", team_name))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    println!("team does not exist");
    println!("creating team");

    // Get the next available team ID
    let next_id = crate::coc::database::get_max_team_id(pool).await? + 1;
    println!("next id: {}", next_id);

    // Use provided handicap or default to 1
    let handicap = handicap.unwrap_or(1);

    if handicap < 1 || handicap > 5 {
        ctx.send(
            poise::CreateReply::default()
                .content("Handicap must be between 1 and 5.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Insert the team into the database
    crate::coc::database::insert_team(pool, next_id, &team_name, handicap).await?;

    // Insert a new set of buildings into the database
    let town_config = &ctx.data().town_config;

    // print the starting level of the buildings
    for (building, config) in &town_config.assets {
        println!("{}: {}", building, config.starting_level);
    }

    // Get the next available building ID
    let mut building_id = crate::coc::database::get_max_building_id(pool).await? + 1;

    // Loop through the buildings and insert them into the database
    for (building, config) in &town_config.assets {
        crate::coc::database::insert_team_building(
            pool,
            building_id,
            next_id,
            &building,
            config.starting_level as i32,
        )
        .await?;
        building_id += 1;
    }

    // This message should be public as it's a positive notification
    ctx.say(format!(
        "Team '{}' created successfully! (ID: {}, Handicap: {})",
        team_name, next_id, handicap
    ))
    .await?;
    Ok(())
}

/// Deletes a team from the database by name
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
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
        ctx.send(
            poise::CreateReply::default()
                .content(format!("No team found with name '{}'", team_name))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Delete any buildings associated with this team first
    crate::coc::database::delete_team_buildings(pool, &team_name).await?;

    // Delete the team from the database
    crate::coc::database::delete_team(pool, &team_name).await?;

    // Use a public notification for successful team deletion
    ctx.say(format!(
        "Team '{}' has been deleted successfully.",
        team_name,
    ))
    .await?;

    Ok(())
}

/// Adds a player to a team
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Check if the player is already on this team
    if let Some(_) = crate::coc::database::get_team_member(pool, team_id, &username).await? {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Player '{}' is already a member of team '{}'",
                    username, team_name
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Get the next available member ID
    let next_id = crate::coc::database::get_max_team_member_id(pool).await? + 1;
    println!("next id: {}", next_id);

    // Add the player to the team
    crate::coc::database::insert_team_member(pool, next_id, team_id, &username).await?;

    // Use public message for successful addition
    ctx.say(format!(
        "Player '{}' has been added to team '{}'!",
        username, team_name
    ))
    .await?;

    Ok(())
}

/// Removes a player from all teams
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
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

    // If player is not in any team, inform the user with ephemeral message
    if team_info.is_none() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("Player '{}' is not a member of any team", username))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Get team name for feedback message
    let (_, team_name) = team_info.unwrap();

    // Remove the player from all teams
    crate::coc::database::delete_team_members(pool, &username).await?;

    // Public message for successful removal
    ctx.say(format!(
        "Successfully removed player '{}' from team: {}",
        username, team_name
    ))
    .await?;

    Ok(())
}

/// Creates an embed message with team resources
#[poise::command(slash_command, prefix_command, owners_only)]
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
            let message_builder = team.create_resource_message().await?;

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

                    ctx.send(
                        poise::CreateReply::default()
                            .content("Resource embed created and recorded successfully!")
                            .ephemeral(true),
                    )
                    .await?;
                }
                Err(why) => {
                    println!("Error sending message: {why:?}");
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!("Error sending message: {}", why))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Lists all resources for a specific team
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Query resources for this team using the database function
    let resources = crate::coc::database::get_team_resources(pool, team_id).await?;

    if resources.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("No resources found for team '{}'", team_name))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Format the results
    let response = resources
        .iter()
        .map(|(id, resource_name, quantity)| {
            format!("• **{}**: {} (Amount: {})", id, resource_name, quantity)
        })
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "**Resources for team '{}':**\n{}",
                team_name, response
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Updates all registered embeds for a specific team
pub async fn update_team_embeds(
    ctx: &serenity::Context,
    data: &Data,
    team_name: &str,
) -> Result<(usize, Vec<String>), Error> {
    // println!("Updating team embeds for team '{}'", team_name);

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
                match embed::get_buildings_embed(data, &team_name).await? {
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
                // println!("Getting resources embed");
                team.make_resource_embed()
            }
            unknown => {
                results.push(format!("Unknown embed variant: {}", unknown));
                continue; // Skip to next embed
            }
        };

        // println!("embed is: {:?}", embed);
        // Try to edit the message
        let result = channel_id
            .edit_message(
                &ctx.http,
                message_id,
                serenity::builder::EditMessage::new().embed(embed),
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

/// Admin Command to Force Insert a resource for a team
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
pub async fn force_insert_resource(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
    #[description = "Name of the resource to insert"] resource_name: String,
    #[description = "Amount of the resource to insert"] quantity: i64,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;
    let patterns = &ctx.data().res_patterns.resource_pattern;

    // Convert inputs to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();
    let item_name = resource_name.to_lowercase();

    // Check if the team exists
    let team_id = match crate::coc::database::get_team_by_name(pool, &team_name).await? {
        Some(id) => id,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Check if item matches resource pattern
    let result = coc::patterns::matches_pattern(&item_name, patterns);

    // Skip this item if it doesn't match any resource pattern
    if !result {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Resource '{}' does not match any known resource pattern",
                    item_name
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // get the category for the item
    let category = coc::patterns::get_resource_category(&item_name, patterns);

    println!("Item match found with category: {}", category);
    // get the modified resource amount
    let quantity =
        coc::database::calculate_resource_total(pool, quantity as i32, team_id, &category).await?;

    // Check if this resource already exists for the team
    let existing_resource = get_resource_quantity_by_name(pool, team_id, &item_name).await?;
    // println!("Existing resource: {:?}", existing_resource);

    match existing_resource {
        Some(resource) => {
            // Update quantity of existing resource
            let new_quantity = resource + quantity as i64;
            update_resource_quantity(pool, team_id, &item_name, new_quantity).await?;

            println!(
                "Updated resource quantity for team '{}': {} x {} (new total: {})",
                team_name, item_name, quantity, new_quantity
            );
        }
        None => {
            // Insert new resource entry
            insert_new_resource(pool, team_id, &item_name, &category, quantity as i64).await?;

            println!(
                "Added new resource for team '{}': {} x {}",
                team_name, item_name, quantity
            );
        }
    }

    // also update any embed resource messages
    let _ = update_team_embeds(ctx.serenity_context(), ctx.data(), &team_name).await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Inserted resource successfully."))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Admin Command to Force Upgrade a building for a team
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
pub async fn force_upgrade_building(
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 2: Check if the building exists in the configuration
    if !town_config.assets.contains_key(&building_name) {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' does not exist in the configuration. Available buildings: {}",
                    building_name,
                    town_config.get_building_types().join(", ")
                ))
                .ephemeral(true),
        )
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Team '{}' doesn't have a '{}' building. Please check the building name.",
                        team_name, building_name
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 4: Check if building is at max level
    let building_config = &town_config.assets[&building_name];
    if current_level as u32 >= building_config.max_level {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' is already at its maximum level ({})!",
                    building_name, current_level
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Step 6: Get upgrade costs using the new enum-based system
    let target_level = current_level + 1;

    // Step 9: Begin transaction to update resources and building level
    let mut tx = pool.begin().await?;

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

    // Step 10: Send success message (public announcement)
    let building_display_name = building_config.name.clone();
    let icon = if !building_config.icon.is_empty() {
        format!("{} ", building_config.icon)
    } else {
        String::new()
    };

    ctx.say(format!(
        "{}**{}** upgraded to level **{}** for team **{}**!\n\n**Resources used:**\n None.",
        icon, building_display_name, target_level, team_name
    ))
    .await?;

    // Step 11: Update any team embeds
    if let Ok((count, _)) =
        update_team_embeds(&ctx.serenity_context(), &ctx.data(), &team_name).await
    {
        if count > 0 {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Updated {} team embeds with the new information.",
                        count
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    // Step 12: Update global embeds if this was a town hall upgrade
    if building_name == "townhall" || building_name == "town_hall" {
        if let Ok((count, _)) = update_global_embeds(
            &ctx.serenity_context(),
            &ctx.data(),
            Some("townhall_ranking"),
        )
        .await
        {
            if count > 0 {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("Updated {} global townhall ranking embeds.", count))
                        .ephemeral(true),
                )
                .await?;
            }
        }
    }

    Ok(())
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 2: Check if the building exists in the configuration
    if !town_config.assets.contains_key(&building_name) {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' does not exist in the configuration. Available buildings: {}",
                    building_name,
                    town_config.get_building_types().join(", ")
                ))
                .ephemeral(true),
        )
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Team '{}' doesn't have a '{}' building. Please check the building name.",
                        team_name, building_name
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 4: Check if building is at max level
    let building_config = &town_config.assets[&building_name];
    if current_level as u32 >= building_config.max_level {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' is already at its maximum level ({})!",
                    building_name, current_level
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Step 6: Get upgrade costs using the new enum-based system
    let target_level = current_level + 1;
    let costs = town_config.get_upgrade_costs(&building_name, target_level as u32);

    if costs.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "No upgrade costs defined for {} at level {}. Please report this to an admin.",
                    building_name, target_level
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Step 7: Check if the team has enough resources
    let mut missing_resources = Vec::new();
    let mut required_resources = Vec::new();

    for cost in &costs {
        match cost {
            // Handle regular resource requirements
            crate::coc::buildings::UpgradeCost::Resource(resource_name, required_amount) => {
                // Get the current amount of this resource
                let resource_query = sqlx::query!(
                    r#"
                    SELECT quantity FROM resources
                    WHERE team_id = $1 AND name = $2
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

            // Handle category-based requirements
            crate::coc::buildings::UpgradeCost::Category(category_name, required_amount) => {
                // Get the total amount of resources in this category
                let total_quantity = crate::coc::database::get_resource_quantity_by_category(
                    pool,
                    team_id.expect("team id should not be null here"),
                    category_name,
                )
                .await?;

                required_resources.push(format!(
                    "`{}` (category): {}",
                    category_name, required_amount
                ));

                if total_quantity < *required_amount as i64 {
                    missing_resources.push(format!(
                        "`{}` (category): have {}/{} ",
                        category_name, total_quantity, required_amount
                    ));
                }
            }
        }
    }

    // Step 8: If missing resources, inform the user and stop
    if !missing_resources.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Insufficient resources to upgrade {} to level {}!\n\n**Required Resources:**\n{}\n\n**Missing Resources:**\n{}",
                    building_name,
                    target_level,
                    required_resources.join("\n"),
                    missing_resources.join("\n")
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Step 9: Begin transaction to update resources and building level
    let mut tx = pool.begin().await?;

    println!("Starting transaction for resource deduction");

    // Deduct resources
    for cost in &costs {
        match cost {
            // Handle regular resource deduction
            crate::coc::buildings::UpgradeCost::Resource(resource_name, amount) => {
                let amount = *amount as i64;
                sqlx::query!(
                    r#"
                    UPDATE resources
                    SET quantity = quantity - $1
                    WHERE team_id = $2 AND name = $3
                    "#,
                    amount,
                    team_id,
                    resource_name
                )
                .execute(&mut *tx)
                .await?;
            }

            // Handle category-based deduction
            crate::coc::buildings::UpgradeCost::Category(category_name, amount) => {
                // Get all resources in this category
                let resources = sqlx::query!(
                    r#"
                    SELECT id as "id: Option<i32>", name, quantity
                    FROM resources
                    WHERE team_id = $1 AND category = $2
                    ORDER BY quantity DESC
                    "#,
                    team_id,
                    category_name
                )
                .fetch_all(&mut *tx)
                .await?;

                // Take the resources proportionally, starting with the largest quantities
                let mut remaining = *amount as i64;

                for resource in resources {
                    if remaining <= 0 {
                        break;
                    }

                    let to_deduct = std::cmp::min(resource.quantity, remaining);

                    if to_deduct > 0 {
                        sqlx::query!(
                            r#"
                            UPDATE resources
                            SET quantity = quantity - $1
                            WHERE id = $2
                            "#,
                            to_deduct,
                            resource.id
                        )
                        .execute(&mut *tx)
                        .await?;

                        remaining -= to_deduct;
                    }
                }

                // If we couldn't deduct enough, report an error and rollback
                if remaining > 0 {
                    // Rollback the transaction
                    tx.rollback().await?;

                    return Err(Error::from(format!(
                        "Could not deduct enough resources from category {}. Transaction rolled back.",
                        category_name
                    )));
                }
            }
        }
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

    // Step 10: Send success message (public announcement)
    let building_display_name = building_config.name.clone();
    let icon = if !building_config.icon.is_empty() {
        format!("{} ", building_config.icon)
    } else {
        String::new()
    };

    // Format the required resources for display in the success message
    let formatted_resources = required_resources.join("\n");

    ctx.say(format!(
        "{}**{}** upgraded to level **{}** for team **{}**!\n\n**Resources used:**\n{}",
        icon, building_display_name, target_level, team_name, formatted_resources
    ))
    .await?;

    // Step 11: Update any team embeds
    if let Ok((count, _)) =
        update_team_embeds(&ctx.serenity_context(), &ctx.data(), &team_name).await
    {
        if count > 0 {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Updated {} team embeds with the new information.",
                        count
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    // Step 12: Update global embeds if this was a town hall upgrade
    if building_name == "townhall" || building_name == "town_hall" {
        if let Ok((count, _)) = update_global_embeds(
            &ctx.serenity_context(),
            &ctx.data(),
            Some("townhall_ranking"),
        )
        .await
        {
            if count > 0 {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("Updated {} global townhall ranking embeds.", count))
                        .ephemeral(true),
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Downgrades a building for a team
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
pub async fn downgrade_building(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
    #[description = "Name of the building to downgrade"] building_name: String,
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No team found with name '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 2: Check if the building exists in the configuration
    if !town_config.assets.contains_key(&building_name) {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' does not exist in the configuration. Available buildings: {}",
                    building_name,
                    town_config.get_building_types().join(", ")
                ))
                .ephemeral(true),
        )
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
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Team '{}' doesn't have a '{}' building. Please check the building name.",
                        team_name, building_name
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Step 4: Check if building is at starting level or not built
    let building_config = &town_config.assets[&building_name];
    if current_level <= building_config.starting_level as i64 {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Building '{}' is already at its starting level ({}) and cannot be downgraded further!",
                    building_name, current_level
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Step 5: Calculate the target level
    let target_level = current_level - 1;

    // Step 6: Downgrade the building
    sqlx::query!(
        r#"
        UPDATE team_buildings
        SET level = level - 1
        WHERE id = $1
        "#,
        building_id
    )
    .execute(pool)
    .await?;

    // Step 7: Send success message (public announcement)
    let building_display_name = building_config.name.clone();
    let icon = if !building_config.icon.is_empty() {
        format!("{} ", building_config.icon)
    } else {
        String::new()
    };

    ctx.say(format!(
        "{}**{}** downgraded to level **{}** for team **{}**!",
        icon, building_display_name, target_level, team_name
    ))
    .await?;

    // Step 8: Update any team embeds
    if let Ok((count, _)) =
        update_team_embeds(&ctx.serenity_context(), &ctx.data(), &team_name).await
    {
        if count > 0 {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "Updated {} team embeds with the new information.",
                        count
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    // Step 12: Update global embeds if this was a town hall upgrade
    if building_name == "townhall" || building_name == "town_hall" {
        if let Ok((count, _)) = update_global_embeds(
            &ctx.serenity_context(),
            &ctx.data(),
            Some("townhall_ranking"),
        )
        .await
        {
            if count > 0 {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("Updated {} global townhall ranking embeds.", count))
                        .ephemeral(true),
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Creates an embed message showing team buildings and their levels
#[poise::command(slash_command, prefix_command, owners_only)]
pub async fn create_buildings_embed(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let data = ctx.data();
    let pool = &data.database;

    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();

    let embeds = match embed::get_buildings_embed(data, &team_name).await? {
        Some(embed) => embed,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No buildings found for team '{}'", team_name))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Create the message builder with the embed
    let message_builder = serenity::builder::CreateMessage::new().embed(embeds);

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
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!("No team found with name '{}'", team_name))
                            .ephemeral(true),
                    )
                    .await?;
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

            ctx.send(
                poise::CreateReply::default()
                    .content("Buildings embed created and recorded successfully!")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(why) => {
            println!("Error sending message: {why:?}");
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("Error sending message: {}", why))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Creates an overview embed showing all teams and their building levels
#[poise::command(slash_command, prefix_command, owners_only)]
pub async fn buildings_overview(ctx: Context<'_>) -> Result<(), Error> {
    // Get data from context
    let data = ctx.data();
    let pool = &data.database;

    // Get the overview embed
    let overview_embed = match embed::get_teams_townhall_levels(data).await? {
        Some(embed) => embed,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("No teams or buildings found to create an overview.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Create the message builder with the embed
    let message_builder = serenity::builder::CreateMessage::new().embed(overview_embed);

    // Send the message
    println!("Sending buildings overview message");
    let msg_result = ctx
        .channel_id()
        .send_message(&ctx.http(), message_builder)
        .await;

    match msg_result {
        Ok(message) => {
            // Record the embed in the global_embeds table
            let channel_id = ctx.channel_id().get() as i64;
            let message_id = message.id.get() as i64;
            let variant = "townhall_ranking".to_string();

            // Check if this variant already exists in global_embeds
            let existing =
                crate::coc::database::get_global_embed_by_variant(pool, &variant).await?;

            if let Some((embed_id, _, _)) = existing {
                println!("Updating existing global embed record");
                // Update existing record
                crate::coc::database::update_global_embed(pool, embed_id, channel_id, message_id)
                    .await?;
            } else {
                println!("Inserting new global embed record");
                // Insert a new global embed record
                crate::coc::database::insert_global_embed(pool, channel_id, &variant, message_id)
                    .await?;
            }

            ctx.send(
                poise::CreateReply::default()
                    .content("Buildings overview has been created and recorded successfully!")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(why) => {
            println!("Error sending message: {why:?}");
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("Error sending message: {}", why))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Updates all registered global embeds
pub async fn update_global_embeds(
    ctx: &serenity::Context,
    data: &Data,
    variant_filter: Option<&str>,
) -> Result<(usize, Vec<String>), Error> {
    println!(
        "Updating global embeds{}",
        variant_filter.map_or(String::new(), |v| format!(" for variant '{}'", v))
    );

    // Get database connection from context data
    let pool = &data.database;

    // Find all global embeds, possibly filtered by variant
    let mut records = Vec::new();

    if let Some(variant) = variant_filter {
        // Filter by variant
        if let Some((embed_id, channel_id, message_id)) =
            crate::coc::database::get_global_embed_by_variant(pool, variant).await?
        {
            records.push((embed_id, channel_id, message_id, variant.to_string()));
        }
    } else {
        // Get all global embeds
        records = crate::coc::database::get_all_global_embeds(pool).await?;
    }

    if records.is_empty() {
        return Ok((0, vec!["No global embeds found".to_string()]));
    }

    let mut updated_count = 0;
    let mut results = Vec::new();

    // Update each embed
    for (embed_id, channel_id_int, message_id_int, variant) in records {
        let channel_id = serenity::ChannelId::new(channel_id_int as u64);
        let message_id = serenity::MessageId::new(message_id_int as u64);

        // Choose embed based on variant
        let embed = match variant.as_str() {
            "townhall_ranking" => {
                // For townhall ranking variant, use the townhall levels embed
                match embed::get_teams_townhall_levels(data).await? {
                    Some(th_embed) => th_embed,
                    None => {
                        results.push("Failed to generate townhall ranking embed".to_string());
                        continue; // Skip to next embed
                    }
                }
            }
            "team_leaderboard" => {
                // For team leaderboard variant (if you add this feature later)
                // match embed::get_team_leaderboard(data).await? ...
                results.push(format!("Unsupported embed variant: {}", variant));
                continue; // Skip to next embed
            }
            "server_statistics" => {
                // For server statistics variant (if you add this feature later)
                // match embed::get_server_statistics(data).await? ...
                results.push(format!("Unsupported embed variant: {}", variant));
                continue; // Skip to next embed
            }
            unknown => {
                results.push(format!("Unknown global embed variant: {}", unknown));
                continue; // Skip to next embed
            }
        };

        // Try to edit the message
        let result = channel_id
            .edit_message(
                &ctx.http,
                message_id,
                serenity::builder::EditMessage::new().embed(embed),
            )
            .await;

        match result {
            Ok(_) => {
                updated_count += 1;
                results.push(format!(
                    "Updated global {} embed in channel {}",
                    variant, channel_id
                ));
            }
            Err(err) => {
                results.push(format!(
                    "Failed to update global {} embed in channel {}: {}",
                    variant, channel_id, err
                ));

                // If message was deleted, remove it from tracking
                if err.to_string().contains("Unknown Message") {
                    crate::coc::database::mark_global_embed_as_deleted(pool, embed_id).await?;

                    results.push(format!(
                        "Marked global message as deleted in database: {} embed in channel {}",
                        variant, channel_id
                    ));
                }
            }
        }
    }

    Ok((updated_count, results))
}
