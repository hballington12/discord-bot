use crate::Error;
use sqlx::SqlitePool;

pub async fn get_user_team(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<(i32, String)>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT tm.team_id as "team_id: i32", t.name as team_name
        FROM team_members tm
        JOIN teams t ON tm.team_id = t.id
        WHERE tm.username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|record| (record.team_id, record.team_name)))
}

pub async fn get_team_armory_level(
    pool: &SqlitePool,
    level: i32,
    team_id: i32,
) -> Result<Option<bool>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            $1 <= acm.max_combat_level AS "has_access: bool"
        FROM teams t
        JOIN team_buildings tb ON t.id = tb.team_id
        JOIN armory_combat_mapping acm ON tb.level = acm.armory_level
        WHERE 
            tb.building_name = 'armory'
            AND t.id = $2
        "#,
        level,
        team_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|record| record.has_access.unwrap_or(false)))
}

pub async fn get_team_slayer_level(
    pool: &SqlitePool,
    level: i32,
    team_id: i32,
) -> Result<Option<bool>, Error> {
    println!(
        "Checking slayer level access for team {} with level {}",
        team_id, level
    );
    let result = sqlx::query!(
        r#"
        SELECT 
            $1 <= smlm.slayer_level AS "has_access: bool"
        FROM teams t
        JOIN team_buildings tb ON t.id = tb.team_id
        JOIN slayer_master_level_mapping smlm ON tb.level = smlm.slayer_master_level
        WHERE 
            tb.building_name = 'slayer_master'
            AND t.id = $2
        "#,
        level,
        team_id
    )
    .fetch_optional(pool)
    .await?;

    println!("Result: {:?}", result);

    Ok(result.map(|record| record.has_access.unwrap_or(false)))
}

pub async fn get_resource_quantity_by_name(
    pool: &SqlitePool,
    team_id: i32,
    item_name: &str,
) -> Result<Option<i64>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT quantity 
        FROM resources
        WHERE team_id = $1 AND name = $2
        "#,
        team_id,
        item_name
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|record| record.quantity))
}

/// Get the total quantity of resources in a specific category for a team
pub async fn get_resource_quantity_by_category(
    pool: &SqlitePool,
    team_id: i32,
    category: &str,
) -> Result<i64, Error> {
    // Using SQLite query to sum quantities and count distinct resources
    let result = sqlx::query!(
        r#"
        SELECT 
            SUM(quantity) as "total_quantity: i64"
        FROM resources
        WHERE team_id = ? AND category = ?
        GROUP BY category
        "#,
        team_id,
        category
    )
    .fetch_optional(pool)
    .await?;

    // If no resources found in the category, return zeros
    match result {
        Some(row) => Ok(row.total_quantity.unwrap_or(0)),
        None => Ok(0),
    }
}

pub async fn update_resource_quantity(
    pool: &SqlitePool,
    team_id: i32,
    item_name: &str,
    quantity: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE resources
        SET quantity = $1
        WHERE team_id = $2 AND name = $3
        "#,
        quantity,
        team_id,
        item_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_new_resource(
    pool: &SqlitePool,
    team_id: i32,
    item_name: &str,
    category: &str,
    quantity: i64,
) -> Result<(), Error> {
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM resources
        "#
    )
    .fetch_one(pool)
    .await?;

    let next_id = max_id_result.max_id.unwrap_or(0) + 1;

    sqlx::query!(
        r#"
        INSERT INTO resources (team_id, id, quantity, name, category)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        team_id,
        next_id,
        quantity,
        item_name,
        category
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_teams(pool: &SqlitePool) -> Result<Vec<(i32, String)>, Error> {
    let teams = sqlx::query!(
        r#"
        SELECT id as "id: i32", name 
        FROM teams
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(teams
        .into_iter()
        .map(|team| (team.id.unwrap(), team.name))
        .collect())
}

pub async fn get_team_by_name(pool: &SqlitePool, team_name: &str) -> Result<Option<i32>, Error> {
    let team = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM teams WHERE name = $1
        "#,
        team_name
    )
    .fetch_optional(pool)
    .await?;

    Ok(team.and_then(|t| t.id.flatten()))
}

pub async fn get_max_team_id(pool: &SqlitePool) -> Result<i32, Error> {
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM teams
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(max_id_result.max_id.unwrap_or(0))
}

pub async fn insert_team(pool: &SqlitePool, team_id: i32, team_name: &str) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO teams (id, name) VALUES ($1, $2)
        "#,
        team_id,
        team_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_max_building_id(pool: &SqlitePool) -> Result<i32, Error> {
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM team_buildings
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(max_id_result.max_id.unwrap_or(0))
}

pub async fn insert_team_building(
    pool: &SqlitePool,
    building_id: i32,
    team_id: i32,
    building_name: &str,
    level: i32,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO team_buildings (id, team_id, building_name, level)
        VALUES ($1, $2, $3, $4)
        "#,
        building_id,
        team_id,
        building_name,
        level
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_team_buildings(pool: &SqlitePool, team_name: &str) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM team_buildings WHERE team_id = (SELECT id FROM teams WHERE name = $1)
        "#,
        team_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_team(pool: &SqlitePool, team_name: &str) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM teams WHERE name = $1
        "#,
        team_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_team_member(
    pool: &SqlitePool,
    team_id: i32,
    username: &str,
) -> Result<Option<i32>, Error> {
    let member = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>" FROM team_members 
        WHERE team_id = $1 AND username = $2
        "#,
        team_id,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(member.and_then(|m| m.id.flatten()))
}

pub async fn get_max_team_member_id(pool: &SqlitePool) -> Result<i32, Error> {
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM team_members
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(max_id_result.max_id.unwrap_or(0))
}

pub async fn insert_team_member(
    pool: &SqlitePool,
    member_id: i32,
    team_id: i32,
    username: &str,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, username)
        VALUES ($1, $2, $3)
        "#,
        member_id,
        team_id,
        username
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_team_members(pool: &SqlitePool, username: &str) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM team_members 
        WHERE username = $1
        "#,
        username
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_team_resources(
    pool: &SqlitePool,
    team_id: i32,
) -> Result<Vec<(i32, String, i64)>, Error> {
    let resources = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", category, quantity 
        FROM resources
        WHERE team_id = $1
        "#,
        team_id
    )
    .fetch_all(pool)
    .await?;

    Ok(resources
        .into_iter()
        .map(|res| (res.id.unwrap().unwrap(), res.category, res.quantity))
        .collect())
}

pub async fn get_team_embeds(
    pool: &SqlitePool,
    team_id: i32,
) -> Result<Vec<(i32, i64, i64, String)>, Error> {
    let embeds = sqlx::query!(
        r#"
        SELECT id as "id: Option<i32>", channel_id, message_id, variant
        FROM team_embeds
        WHERE team_id = $1 AND message_id IS NOT NULL
        "#,
        team_id
    )
    .fetch_all(pool)
    .await?;

    Ok(embeds
        .into_iter()
        .map(|embed| {
            (
                embed.id.unwrap().unwrap(),
                embed.channel_id,
                embed.message_id.unwrap(),
                embed.variant,
            )
        })
        .collect())
}

pub async fn update_team_embed(
    pool: &SqlitePool,
    embed_id: i32,
    channel_id: i64,
    message_id: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE team_embeds 
        SET channel_id = $1, message_id = $2
        WHERE id = $3
        "#,
        channel_id,
        message_id,
        embed_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_team_embed(
    pool: &SqlitePool,
    team_id: i32,
    channel_id: i64,
    variant: &str,
    message_id: i64,
) -> Result<(), Error> {
    let max_id_result = sqlx::query!(
        r#"
        SELECT MAX(id) as "max_id: i32" FROM team_embeds
        "#
    )
    .fetch_one(pool)
    .await?;

    let next_id = max_id_result.max_id.unwrap_or(0) + 1;

    sqlx::query!(
        r#"
        INSERT INTO team_embeds (id, team_id, channel_id, variant, message_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        next_id,
        team_id,
        channel_id,
        variant,
        message_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_embed_as_deleted(pool: &SqlitePool, embed_id: i32) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE team_embeds
        SET message_id = NULL
        WHERE id = $1
        "#,
        embed_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get a team's multiplier for a specific resource category
pub async fn get_team_resource_multiplier(
    pool: &SqlitePool,
    team_id: i32,
    resource_category: &str,
) -> Result<f64, Error> {
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(MAX(multiplier), 1.0) as "multiplier: f64"
        FROM team_resource_multipliers
        WHERE team_id = ? AND resource_category = ?
        "#,
        team_id,
        resource_category
    )
    .fetch_one(pool)
    .await?;

    Ok(result.multiplier)
}

/// Get a team's flat bonus for a specific resource category
pub async fn get_team_resource_flat_bonus(
    pool: &SqlitePool,
    team_id: i32,
    resource_category: &str,
) -> Result<i32, Error> {
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(MAX(flat_bonus), 0) as "bonus: i32"
        FROM team_resource_multipliers
        WHERE team_id = ? AND resource_category = ?
        "#,
        team_id,
        resource_category
    )
    .fetch_one(pool)
    .await?;

    Ok(result.bonus)
}

/// Calculate total resources after applying multiplier and bonus
pub async fn calculate_resource_total(
    pool: &SqlitePool,
    base_amount: i32,
    team_id: i32,
    resource_category: &str,
) -> Result<i32, Error> {
    let multiplier = get_team_resource_multiplier(pool, team_id, resource_category).await?;
    let flat_bonus = get_team_resource_flat_bonus(pool, team_id, resource_category).await?;

    // Apply multiplier first, then add flat bonus: floor(base_amount * multiplier) + flat_bonus
    let with_multiplier = (base_amount as f64 * multiplier).floor() as i32;
    let total = with_multiplier + flat_bonus;

    println!(
        "Calculated resource total: {} * {} + {} = {}",
        base_amount, multiplier, flat_bonus, total
    );

    Ok(total)
}

/// Get the level of a specific building for a team
/// Returns the building's level, or 0 if the building doesn't exist
pub async fn get_team_building_level(
    pool: &SqlitePool,
    team_id: i32,
    building_name: &str,
) -> Result<i32, Error> {
    let result = sqlx::query!(
        r#"
        SELECT level as "level: i32"
        FROM team_buildings
        WHERE team_id = ? AND building_name = ?
        "#,
        team_id,
        building_name
    )
    .fetch_optional(pool)
    .await?;

    // Return the level if found, otherwise return 0 (indicating the building doesn't exist)
    match result {
        Some(row) => Ok(row.level),
        None => Ok(0), // Default level 0 means building doesn't exist
    }
}
