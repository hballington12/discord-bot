mod coc;
mod commands;
mod dink;

use std::{
    env::{self, var},
    sync::Arc,
    time::Duration,
};

use poise::serenity_prelude as serenity;
use sqlx::{Database, SqlitePool};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// to be customised later
pub struct Data {
    dink_channel_id: u64,
    database: sqlx::SqlitePool,
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
    ctx: &serenity::Context,
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
                dink::handle_message(new_message); // parse dink messages
            }
        }
        _ => {}
    }
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
            coc::commands::bage(),
            coc::commands::add_player(),
            list_teams(), // Add the new command here
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

                // // Get the project root directory from CARGO_MANIFEST_DIR
                // let manifest_dir =
                //     std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
                // let database_url = var("DATABASE_URL").expect("Failed to get database url");

                // let database_url = "db/coc.db";

                // Initiate a connection to the database file, creating the file if required.
                // let database = sqlx::sqlite::SqlitePoolOptions::new()
                //     .max_connections(5)
                //     .connect_with(
                //         sqlx::sqlite::SqliteConnectOptions::new()
                //     )
                //     .await
                //     .expect("Couldn't connect to database");

                let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

                Ok(Data {
                    dink_channel_id,
                    database: pool,
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
