mod coc;
mod commands;
mod dink;

use std::{
    env::{self, var},
    sync::Arc,
    time::Duration,
};

use ::serenity::all::{CreateEmbedFooter, CreateMessage, EditMessage};
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// to be customised as needed
pub struct Data {
    dink_channel_id: u64,
    database: sqlx::SqlitePool,
    res_patterns: coc::patterns::PatternConfig,
    status_message: tokio::sync::Mutex<Option<(serenity::ChannelId, serenity::MessageId)>>,
} // User data, which is stored and accessible in all command invocations

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            // Use the stored channel ID
            if new_message.channel_id.get() == data.dink_channel_id {
                println!("responding to message in dink channel");
                if let Err(e) = dink::handle_message(data, new_message).await {
                    println!("Error handling dink message: {}", e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Creates or updates a status embed with live information
async fn update_status_embed(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    data: &Data,
    message_id: Option<serenity::MessageId>,
) -> Result<serenity::MessageId, Error> {
    // Query latest information from database
    let pool = &data.database;

    // Get top teams information
    let teams = sqlx::query!(
        r#"
        SELECT 
            t.name, 
            COUNT(DISTINCT r.resource_name) as resource_count,
            SUM(r.quantity) as total_resources
        FROM teams t
        LEFT JOIN resources r ON t.id = r.team_id
        GROUP BY t.id
        ORDER BY total_resources DESC
        LIMIT 5
        "#
    )
    .fetch_all(pool)
    .await?;

    // Get total resource counts
    let resource_stats = sqlx::query!(
        r#"
        SELECT 
            resource_name, 
            SUM(quantity) as total
        FROM resources
        GROUP BY resource_name
        ORDER BY total DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    // Build the embed
    let mut embed = serenity::CreateEmbed::default()
        .title("CoC Resource Dashboard")
        .description("Live resource tracking for teams")
        .color(0x3498db) // Blue color
        .timestamp(chrono::Utc::now());

    // Add team rankings field
    if !teams.is_empty() {
        let team_text = teams
            .iter()
            .map(|t| {
                format!(
                    "**{}**: {} resources ({} types)",
                    t.name,
                    t.total_resources.unwrap_or(0),
                    t.resource_count
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        embed = embed.field("Team Rankings", team_text, false);
    } else {
        embed = embed.field("Team Rankings", "No teams found", false);
    }

    // Add top resources field
    if !resource_stats.is_empty() {
        let resources_text = resource_stats
            .iter()
            .map(|r| format!("**{}**: {}", r.resource_name, r.total))
            .collect::<Vec<_>>()
            .join("\n");

        embed = embed.field("Most Collected Resources", resources_text, false);
    } else {
        embed = embed.field(
            "Most Collected Resources",
            "No resources collected yet",
            false,
        );
    }

    // Create or update the message
    if let Some(msg_id) = message_id {
        // Update existing message
        let builder = EditMessage::new().content("hello");
        let mut embed_message = channel_id.message(ctx, msg_id).await?;
        embed_message.edit(ctx, builder).await?;
        Ok(msg_id)
    } else {
        // Create new message
        let builder = CreateMessage::new().content("hello");
        let result = channel_id.send_message(ctx, builder).await?;
        let message_id = result.id;
        Ok(message_id)
    }
}

/// Creates or refreshes the resource status dashboard
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn dashboard(
    ctx: Context<'_>,
    #[description = "Channel to post the dashboard in (defaults to current channel)"]
    channel: Option<serenity::GuildChannel>,
) -> Result<(), Error> {
    let channel_id = channel.map(|c| c.id).unwrap_or_else(|| ctx.channel_id());

    // Update the status embed and store the message ID
    let message_id = {
        let mut status_lock = ctx.data().status_message.lock().await;
        let existing_id = status_lock.as_ref().map(|(_, id)| *id);

        let new_id =
            update_status_embed(ctx.serenity_context(), channel_id, ctx.data(), existing_id)
                .await?;

        // Store the new message info
        *status_lock = Some((channel_id, new_id));
        new_id
    };

    ctx.say(format!(
        "Dashboard created/updated! Message ID: {}",
        message_id
    ))
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::age(),
            coc::commands::list_teams(), // Add the new command here
            coc::commands::add_player(),
            coc::commands::add_team(),
            coc::commands::remove_team(),
            coc::commands::remove_player(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot,"),
                poise::Prefix::Literal("hey bot"),
            ],
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            let _ = Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
            });
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Parse channel ID once during setup
                let dink_channel_id = var("DINK_UPDATES_CHANNEL_ID")
                    .expect("Missing `DINK_UPDATES_CHANNEL_ID` env var")
                    .parse::<u64>()
                    .expect("DINK_UPDATES_CHANNEL_ID must be a valid u64");

                // connect to database
                let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

                // load the pattern config
                let res_patterns = coc::patterns::load_res_patterns();

                Ok(Data {
                    dink_channel_id,
                    database: pool,
                    res_patterns: res_patterns,
                    status_message: tokio::sync::Mutex::new(None),
                })
            })
        })
        .options(options)
        .build();

    dotenv::dotenv().expect("Failed to load .env file");
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
