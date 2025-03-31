mod coc;
mod commands;
mod dink;
mod utils;
mod webhook;

use std::{
    env::{self, var},
    sync::Arc,
    time::Duration,
};

use ::serenity::all::GatewayIntents;
use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;
use tokio::sync::Mutex as TokioMutex;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    dink_channel_id: u64,
    database: sqlx::SqlitePool,
    res_patterns: coc::patterns::PatternConfig,
    town_config: coc::buildings::TownConfig,
    bestiary: coc::bestiary::Bestiary,
    status_message: tokio::sync::Mutex<Option<(serenity::ChannelId, serenity::MessageId)>>,
    webhook_receiver: TokioMutex<Option<webhook::WebhookReceiver>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
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

            let ctx_clone = ctx.clone();

            let mut receiver_guard = data.webhook_receiver.lock().await;
            if let Some(mut receiver) = receiver_guard.take() {
                drop(receiver_guard);

                while let Some(payload) = receiver.recv().await {
                    // println!("Processing webhook: {:?}", payload);
                    if let Err(e) = process_webhook(&ctx_clone, &data, &payload).await {
                        eprintln!("Error processing webhook: {}", e);
                    }
                }
            }
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.channel_id.get() == data.dink_channel_id {
                if let Err(e) = dink::handle_message(ctx, data, new_message).await {
                    println!("Error handling dink message: {}", e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn process_webhook(
    ctx: &serenity::Context,
    data: &Data,
    payload: &webhook::WebhookPayload,
) -> Result<(), Error> {
    let channel_id = serenity::ChannelId::new(data.dink_channel_id);
    // println!(
    //     "Processing webhook from {}: {}",
    //     payload.playerName, payload.r#type
    // );

    // Process each embed in the payload
    for embed in &payload.embeds {
        let description = &embed.description;

        // println!("Processing embed description: {}", description);

        // Parse the loot text using your existing function
        match dink::parse_loot_text(description) {
            Ok(drop) => {
                println!(
                    "Processing drop: User: {}, Source: {}, Items: {:?}",
                    drop.user, drop.source, drop.loots
                );

                // Process the drop using your existing function
                if let Err(e) = dink::process_drop(ctx, data, drop).await {
                    eprintln!("Error processing drop: {}", e);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Failed to parse loot text: {}", e);

                // Send a simple message with the raw embed info if parsing fails
                let message = format!(
                    "**New Event from {}**\nType: {}\n\n{}",
                    payload.playerName, payload.r#type, description
                );

                if let Err(send_err) = channel_id.say(&ctx.http, message).await {
                    eprintln!("Error sending message: {}", send_err);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let webhook_port = var("WEBHOOK_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("WEBHOOK_PORT must be a valid port number");

    let (_webhook_sender, webhook_receiver) = webhook::start_webhook_server(webhook_port).await;

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::age(),
            coc::commands::list_teams(),
            coc::commands::add_player(),
            coc::commands::add_team(),
            coc::commands::remove_team(),
            coc::commands::remove_player(),
            coc::commands::create_resource_embed(),
            coc::commands::list_team_resources(),
            coc::commands::upgrade_building(),
            coc::commands::create_buildings_embed(),
            coc::commands::downgrade_building(),
            coc::commands::helper::lookup_resource(),
            coc::commands::helper::lookup_category(),
            coc::commands::buildings_overview(),
            coc::commands::force_upgrade_building(),
            coc::commands::force_insert_resource(),
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
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
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

                let dink_channel_id = var("DINK_UPDATES_CHANNEL_ID")
                    .expect("Missing `DINK_UPDATES_CHANNEL_ID` env var")
                    .parse::<u64>()
                    .expect("DINK_UPDATES_CHANNEL_ID must be a valid u64");

                // Configure SQLite pool with optimized settings for high concurrency
                let pool_options = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
                    .max_connections(20) // Increase max connections in the pool
                    .min_connections(5) // Keep some connections ready
                    .idle_timeout(std::time::Duration::from_secs(30))
                    .acquire_timeout(std::time::Duration::from_secs(30));

                let pool = pool_options
                    .connect_with(
                        sqlx::sqlite::SqliteConnectOptions::new()
                            .filename(&env::var("DATABASE_URL")?)
                            .create_if_missing(true)
                            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // Write-Ahead Logging for better concurrency
                            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal) // Balance between safety and performance
                            .foreign_keys(true)
                            .busy_timeout(std::time::Duration::from_secs(30)), // Longer timeout for busy connections
                    )
                    .await?;

                let res_patterns = coc::patterns::load_res_patterns();

                let town_config =
                    coc::buildings::init_assets().expect("could not load town config");

                let bestiary = coc::bestiary::init_bestiary().expect("could not load bestiary");

                Ok(Data {
                    dink_channel_id,
                    database: pool,
                    res_patterns,
                    town_config,
                    bestiary,
                    status_message: tokio::sync::Mutex::new(None),
                    webhook_receiver: TokioMutex::new(Some(webhook_receiver)),
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
