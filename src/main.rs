mod coc;
mod commands;
mod dink;

use std::collections::{HashMap, HashSet};
use std::env;

use serenity::all::Message;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler {
    dink_updates_channel_id: u64,
    interesting_items: HashSet<String>,
    username_to_team: HashMap<String, coc::TeamInfo>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![commands::ping::register()])
            .await;

        println!("I now have the following guild slash commands: {commands:#?}");
    }

    async fn message(&self, ctx: Context, message: Message) {
        // Check if the message is from the dink updates channel
        if message.channel_id == self.dink_updates_channel_id {
            // Parse the message using our function
            if let Some(parsed) = dink::parse_dink_update(&message.content) {
                println!("Username: {}", parsed.username);
                println!("Loot: {}", parsed.loot_string);
                println!("Source: {}", parsed.source);

                // Print parsed items
                println!("Parsed items:");
                for (item, quantity) in &parsed.items {
                    println!("  {} x {}", quantity, item);
                }

                // Check for interesting items
                let mut found_interesting_items = Vec::new();
                for (item, quantity) in &parsed.items {
                    if self.interesting_items.contains(item) {
                        println!("found interesting item: {}", item);
                        found_interesting_items.push((item, *quantity));
                    }
                }

                // Look up team name from username
                if let Some(team_info) = self.username_to_team.get(&parsed.username) {
                    println!(
                        "User {} belongs to team {} (ID: {})",
                        parsed.username, team_info.name, team_info.id
                    );
                } else {
                    println!("User {} does not belong to any known team", parsed.username);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Parse the dink updates channel ID once
    let dink_updates_channel_id = env::var("DINK_UPDATES_CHANNEL_ID")
        .expect("Expected DINK_UPDATES_CHANNEL_ID in environment")
        .parse()
        .expect("DINK_UPDATES_CHANNEL_ID must be a valid ID");

    // Build our client.
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler {
        dink_updates_channel_id,
        interesting_items: coc::setup_interesting_items(),
        username_to_team: coc::setup_username_teams(),
    })
    .await
    .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
