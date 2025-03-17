use crate::{Data, Error};

use crate::coc::get_team;
use crate::Context;
use ::serenity::all::CreateEmbed;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

/// Looks up a specific resource for a team
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn lookup_resource(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
    #[description = "Name of the resource to look up"] resource_name: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert inputs to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();
    let resource_name = resource_name.to_lowercase();

    // Step 1: Check if the team exists
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

    // Step 2: Query for the specific resource
    let quantity =
        match crate::coc::database::get_resource_quantity_by_name(pool, team_id, &resource_name)
            .await?
        {
            Some(amount) => amount,
            None => 0, // Resource doesn't exist, default to 0
        };

    // Step 3: Format and send the response as ephemeral message
    let emoji = match resource_name.as_str() {
        "coins" | "gp" => "💰",
        "runes" => "🔮",
        r if r.contains("ore") => "⛏️",
        r if r.contains("log") => "🪵",
        r if r.contains("fish") => "🐟",
        r if r.contains("gem") => "💎",
        _ => "📦", // Default emoji
    };

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "{} Team **{}** have **{}** `{}`",
                emoji, team_name, quantity, resource_name,
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Looks up total resources in a category for a team
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn lookup_category(
    ctx: Context<'_>,
    #[description = "Name of the team"] team_name: String,
    #[description = "Category to look up (mining, fishing, etc.)"] category: String,
) -> Result<(), Error> {
    // Get database connection from context data
    let pool = &ctx.data().database;

    // Convert inputs to lowercase for consistent lookups
    let team_name = team_name.to_lowercase();
    let category = category.to_lowercase();

    // Step 1: Check if the team exists
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

    // Step 2: Query for resources in the specified category using our function
    let total_quantity =
        crate::coc::database::get_resource_quantity_by_category(pool, team_id, &category).await?;

    // Step 3: Format and send the response as ephemeral message
    // Select an emoji based on category
    let emoji = match category.as_str() {
        "mining" => "⛏️",
        "woodcutting" => "🪓",
        "fishing" => "🎣",
        "farming" => "🌾",
        "herblore" => "🌿",
        "hunting" => "🏹",
        "crafting" => "🔨",
        "currency" => "💰",
        "gems" => "💎",
        "runes" => "🔮",
        "combat" => "⚔️",
        _ => "📦", // Default emoji
    };

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "{} Team **{}** has **{}** total items in the **{}** category ",
                emoji, team_name, total_quantity, category
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
