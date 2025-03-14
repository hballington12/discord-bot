use crate::{Context, Error};

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
