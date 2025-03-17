use std::collections::{HashMap, HashSet};

use crate::{Context, Data, Error};
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;

pub mod bestiary;
pub mod buildings;
pub mod commands;
pub mod database;
pub mod patterns;

/// Internal struct for a team.
pub struct Team {
    id: i32,
    name: String,
    resources: HashMap<String, i64>,
}

impl Team {
    /// Creates an embed containing team information and resources
    pub fn make_resource_embed(&self) -> CreateEmbed {
        // Create the title with team name
        let title = format!("{} Team Resources", self.name);

        // Create footer with team ID
        let footer = CreateEmbedFooter::new(format!("Team ID: {}", self.id));

        // Start building the embed
        let mut embed = CreateEmbed::new()
            .title(title)
            .description(format!("Resource inventory for team **{}**", self.name))
            .footer(footer)
            .timestamp(Timestamp::now());

        // Sort resources by name for consistent display
        let mut sorted_resources: Vec<(&String, &i64)> = self.resources.iter().collect();
        sorted_resources.sort_by(|a, b| a.0.cmp(b.0));

        if !self.resources.is_empty() {
            // Create a table-like format for resources
            let mut resources_table = String::new();

            // Find the longest resource name for padding
            let max_name_length = sorted_resources
                .iter()
                .map(|(name, _)| name.len())
                .max()
                .unwrap_or(0);

            // Build the table string
            for (name, quantity) in sorted_resources {
                resources_table.push_str(&format!(
                    "`{:<width$}` : **{}**\n",
                    name,
                    quantity,
                    width = max_name_length
                ));
            }

            // Add the resources table as a single field
            embed = embed.field("Resources", resources_table, false);
        } else {
            // If there are no resources, add a note
            embed = embed.field("No Resources", "This team has no resources yet.", false);
        }

        // Add total count
        let total_resources = self.resources.len();
        let total_quantity: i64 = self.resources.values().sum();
        embed = embed.field(
            "Summary",
            format!(
                "**{}** resource types\n**{}** total items",
                total_resources, total_quantity
            ),
            false,
        );

        embed
    }

    /// Creates a message builder with the team embed
    pub async fn create_resource_message(&self) -> Result<CreateMessage, Error> {
        let embed = self.make_resource_embed();

        // Create the message with the embed
        let message = CreateMessage::new()
            .content(format!("Team Info: **{}**", self.name))
            .embed(embed);

        Ok(message)
    }
}

/// Construct team from query
pub async fn get_team(data: &Data, team_name: &String) -> Result<Option<Team>, Error> {
    // Get database connection from context data
    let pool = &data.database;

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
            println!("no team found with name '{}'", team_name);
            return Ok(None);
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

    // Build the resource map
    let mut resource_map = HashMap::new();
    for res in resources.iter() {
        resource_map.insert(res.resource_name.clone(), res.quantity);
    }

    // Create the Team struct
    let team = Team {
        id: team_id.unwrap(),
        name: team_name,
        resources: resource_map,
    };

    Ok(Some(team))
}

// // Add this struct to track team information
// #[derive(Clone)]
// pub struct TeamInfo {
//     pub id: u32,
//     pub name: String,
// }

// // Create a function to initialize the set of interesting items
// pub fn setup_interesting_items() -> HashSet<String> {
//     let mut items = HashSet::new();

//     // Add items that we're interested in
//     items.insert("Coal".to_string());
//     items.insert("Bones".to_string());
//     items.insert("Water rune".to_string());
//     items.insert("Nature rune".to_string());
//     items.insert("Death rune".to_string());
//     items.insert("Blood rune".to_string());
//     items.insert("Law rune".to_string());
//     items.insert("Soul rune".to_string());

//     // Add more items as needed

//     items
// }

// // Create a function to initialize the username to team mapping
// pub fn setup_username_teams() -> HashMap<String, TeamInfo> {
//     let mut username_map = HashMap::new();

//     // Team 1
//     let team1 = TeamInfo {
//         id: 1,
//         name: "Dragon Hunters".to_string(),
//     };

//     username_map.insert("Solo H".to_string(), team1.clone());
//     username_map.insert("Group Chrog".to_string(), team1.clone());
//     username_map.insert("Magic Pro".to_string(), team1);

//     // Team 2
//     let team2 = TeamInfo {
//         id: 2,
//         name: "Goblin Squad".to_string(),
//     };

//     username_map.insert("Goblin_1".to_string(), team2.clone());
//     username_map.insert("Goblin_2".to_string(), team2.clone());
//     username_map.insert("Goblin_King".to_string(), team2);

//     // Add more teams and usernames as needed

//     username_map
// }
