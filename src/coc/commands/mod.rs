use crate::coc::get_team;
use crate::{Context, Error};

use ::serenity::builder;
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;

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

    // Query all teams from the database
    let teams = sqlx::query!(
        r#"
        SELECT id, name 
        FROM teams
        "#
    )
    .fetch_all(pool)
    .await?;

    if teams.is_empty() {
        ctx.say("No teams found in the database.").await?;
        return Ok(());
    }

    // Format the results
    let response = teams
        .iter()
        .map(|team| format!("• {} (ID: {:?})", team.name, team.id))
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

    // Check if team with this name already exists
    let existing_team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    if existing_team.is_some() {
        ctx.say(format!("Team '{}' already exists!", team_name))
            .await?;
        return Ok(());
    }

    println!("team does not exist");
    println!("creating team");

    // Get the last inserted team ID
    // Get the max ID from the teams table
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM teams
        "#
    )
    .fetch_one(pool)
    .await?;

    // The first inserted team will have ID 1, otherwise we increment the max ID
    let next_id = max_id_result.max_id.unwrap_or(0) + 1;
    println!("next id: {}", next_id);

    // Insert the team into the database
    let _ = sqlx::query!(
        r#"
        INSERT INTO teams (id, name) VALUES ($1, $2)
        "#,
        next_id,
        team_name
    )
    .fetch_optional(pool)
    .await?;

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
    let existing_team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", name FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    // If the team doesn't exist, inform the user and return early
    if let None = existing_team {
        ctx.say(format!("No team found with name '{}'", team_name))
            .await?;
        return Ok(());
    }

    // Delete the team from the database
    sqlx::query!(
        r#"
        DELETE FROM teams WHERE name = $1
        "#,
        team_name
    )
    .execute(pool)
    .await?;

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
    let team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    // If the team doesn't exist, inform the user and return early
    let team_id = match team {
        Some(team) => team.id.ok_or_else(|| Error::from("Team ID is null"))?,
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Check if the player is already on this team
    let existing_member = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM team_members 
        WHERE team_id = $1 AND username = $2
        "#,
        team_id,
        username
    )
    .fetch_optional(pool)
    .await?;

    if existing_member.is_some() {
        ctx.say(format!(
            "Player '{}' is already a member of team '{}'",
            username, team_name
        ))
        .await?;
        return Ok(());
    }

    // Get the last inserted team ID
    // Get the max ID from the teams table
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM team_members
        "#
    )
    .fetch_one(pool)
    .await?;

    // The first inserted team will have ID 1, otherwise we increment the max ID
    let next_id = max_id_result.max_id.unwrap_or(0) + 1;
    println!("next id: {}", next_id);

    // Add the player to the team
    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, username)
        VALUES ($1, $2, $3)
        "#,
        next_id,
        team_id,
        username
    )
    .execute(pool)
    .await?;

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
    let existing_member = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", team_id as "team_id: i32" 
        FROM team_members 
        WHERE username = $1
        "#,
        username
    )
    .fetch_all(pool)
    .await?;

    // If player is not in any team, inform the user
    if existing_member.is_empty() {
        ctx.say(format!("Player '{}' is not a member of any team", username))
            .await?;
        return Ok(());
    }

    // Get team names for feedback message
    let mut team_names = Vec::new();
    for member in &existing_member {
        let team = sqlx::query!(
            r#"
            SELECT name FROM teams WHERE id = $1
            "#,
            member.team_id
        )
        .fetch_one(pool)
        .await?;

        team_names.push(team.name);
    }

    // Remove the player from all teams
    sqlx::query!(
        r#"
        DELETE FROM team_members 
        WHERE username = $1
        "#,
        username
    )
    .execute(pool)
    .await?;

    ctx.say(format!(
        "Successfully removed player '{}' from {} team(s): {}",
        username,
        team_names.len(),
        team_names.join(", ")
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
    let pool = &ctx.data().database;

    // Get team data from database
    let team_opt = get_team(ctx, &team_name).await?;

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
                    let existing = sqlx::query!(
                        r#"
                        SELECT id as "id: Option<i32>" FROM team_embeds 
                        WHERE team_id = $1 AND variant = $2
                        "#,
                        team.id,
                        variant
                    )
                    .fetch_optional(pool)
                    .await?;

                    // Update existing or insert new record
                    if let Some(record) = existing {
                        // Update existing record
                        sqlx::query!(
                            r#"
                            UPDATE team_embeds 
                            SET channel_id = $1, message_id = $2
                            WHERE team_id = $3 AND variant = $4
                            "#,
                            channel_id,
                            message_id,
                            team.id,
                            variant
                        )
                        .execute(pool)
                        .await?;
                    } else {
                        // Insert a new embed record
                        sqlx::query!(
                            r#"
                            INSERT INTO team_embeds (team_id, channel_id, variant, message_id)
                            VALUES ($1, $2, $3, $4)
                            "#,
                            team.id,
                            channel_id,
                            variant,
                            message_id
                        )
                        .execute(pool)
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

    // Check if the team exists and get its ID
    let team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    // If the team doesn't exist, inform the user and return early
    let team_id = match team {
        Some(team) => team.id.ok_or_else(|| Error::from("Team ID is null"))?,
        None => {
            ctx.say(format!("No team found with name '{}'", team_name))
                .await?;
            return Ok(());
        }
    };

    // Query resources for this team
    let resources = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", resource_name, quantity 
        FROM resources
        WHERE team_id = $1
        "#,
        team_id
    )
    .fetch_all(pool)
    .await?;

    if resources.is_empty() {
        ctx.say(format!("No resources found for team '{}'", team_name))
            .await?;
        return Ok(());
    }

    // Format the results
    let response = resources
        .iter()
        .map(|res| {
            format!(
                "• **{:?}**: {} (Amount: {})",
                res.id, res.resource_name, res.quantity
            )
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
    ctx: &Context<'_>,
    team_name: &str,
) -> Result<(usize, Vec<String>), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert team name to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();

    // Get team data from database
    let team_opt = get_team(*ctx, &team_name.to_string()).await?;

    let team = match team_opt {
        Some(team) => team,
        None => return Ok((0, vec!["Team not found".to_string()])), // No team found
    };

    // Find all embeds for this team
    let records = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", channel_id, message_id, variant
        FROM team_embeds
        WHERE team_id = $1 AND message_id IS NOT NULL
        "#,
        team.id
    )
    .fetch_all(pool)
    .await?;

    if records.is_empty() {
        return Ok((0, vec!["No embeds found for this team".to_string()]));
    }

    let mut updated_count = 0;
    let mut results = Vec::new();

    // Update each embed
    for record in records {
        let channel_id_int = record.channel_id;
        let message_id_int = record.message_id.expect("msg_id is null");
        let channel_id = serenity::ChannelId::new(channel_id_int as u64);
        let message_id = serenity::MessageId::new(message_id_int as u64);

        let embed = team.create_embed();

        // Try to edit the message
        let result = channel_id
            .edit_message(
                &ctx.http(),
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
                    record.variant, channel_id
                ));
            }
            Err(err) => {
                results.push(format!(
                    "Failed to update {} embed in channel {}: {}",
                    record.variant, channel_id, err
                ));

                // If message was deleted, remove it from tracking
                if err.to_string().contains("Unknown Message") {
                    sqlx::query!(
                        r#"
                        UPDATE team_embeds
                        SET message_id = NULL
                        WHERE id = $1
                        "#,
                        record.id
                    )
                    .execute(pool)
                    .await?;

                    results.push(format!(
                        "Marked message as deleted in database: {} embed in channel {}",
                        record.variant, channel_id
                    ));
                }
            }
        }
    }

    Ok((updated_count, results))
}

/// Command to update all embeds for a team
#[poise::command(slash_command, prefix_command)]
pub async fn update_embeds(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
) -> Result<(), Error> {
    let (count, results) = update_team_embeds(&ctx, &team_name).await?;

    let summary = if count > 0 {
        format!(
            "Successfully updated {} embeds for team '{}'",
            count, team_name
        )
    } else {
        format!("No embeds were updated for team '{}'", team_name)
    };

    // Join the results with line breaks
    let details = results.join("\n");

    // Send the response
    ctx.say(format!("{}\n\nDetails:\n{}", summary, details))
        .await?;

    Ok(())
}
