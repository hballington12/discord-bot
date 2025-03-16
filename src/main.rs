mod coc;
mod commands;
mod dink;
mod utils;

use std::{
    env::{self, var},
    sync::Arc,
    time::Duration,
};

use ::serenity::all::GatewayIntents;
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// to be customised as needed
pub struct Data {
    dink_channel_id: u64,
    database: sqlx::SqlitePool,
    res_patterns: coc::patterns::PatternConfig,
    town_config: coc::buildings::TownConfig,
    bestiary: coc::bestiary::Bestiary,
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
                println!("responding to message in dink channel");
                if let Err(e) = dink::handle_message(ctx, data, new_message).await {
                    println!("Error handling dink message: {}", e);
                }
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
            coc::commands::list_teams(), // Add the new command here
            coc::commands::add_player(),
            coc::commands::add_team(),
            coc::commands::remove_team(),
            coc::commands::remove_player(),
            coc::commands::create_resource_embed(),
            coc::commands::list_team_resources(),
            coc::commands::upgrade_building(),
            coc::commands::create_buildings_embed(),
            commands::simple_embed(),
            commands::edit_embed(),
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

                // load town config
                let town_config =
                    coc::buildings::init_assets().expect("could not load town config");

                // load bestiary
                let bestiary = coc::bestiary::init_bestiary().expect("could not load bestiary");

                Ok(Data {
                    dink_channel_id,
                    database: pool,
                    res_patterns,
                    town_config,
                    bestiary,
                    status_message: tokio::sync::Mutex::new(None),
                })
            })
        })
        .options(options)
        .build();

    dotenv::dotenv().expect("Failed to load .env file");
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
