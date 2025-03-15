use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;
use serenity::Color;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Seleced user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// Examble embed commands
///

// Example command using the embed
#[poise::command(slash_command, prefix_command)]
pub async fn simple_embed(ctx: Context<'_>) -> Result<(), Error> {
    let footer = CreateEmbedFooter::new("This is a footer");
    let embed = CreateEmbed::new()
        .title("This is a title")
        .description("This is a description")
        .image("attachment://ferris_eyes.png")
        .fields(vec![
            ("This is the first field", "This is a field body", true),
            ("This is the second field", "Both fields are inline", true),
        ])
        .field(
            "This is the third field",
            "This is not an inline field",
            false,
        )
        .footer(footer)
        // Add a timestamp for the current time
        // This also accepts a rfc3339 Timestamp
        .timestamp(Timestamp::now());

    println!("Creating message");
    let builder = CreateMessage::new()
        .content("Hello, World!")
        .embed(embed)
        .add_file(CreateAttachment::path("./ferris_eyes.png").await.unwrap());

    println!("Sending message");
    let msg = ctx.channel_id().send_message(&ctx.http(), builder).await;
    println!("Message sent");

    if let Err(why) = msg {
        println!("Error sending message: {why:?}");
    }

    ctx.say("done.").await?;

    Ok(())
}

/// Edit an existing embed message by ID
#[poise::command(slash_command, prefix_command)]
pub async fn edit_embed(ctx: Context<'_>) -> Result<(), Error> {
    // Hardcoded message ID - replace this with your actual message ID
    let message_id = serenity::MessageId::new(1350410790129238067); // Replace with your message ID
    let channel_id = ctx.channel_id();

    let footer = CreateEmbedFooter::new("This footer was edited");
    let embed = CreateEmbed::new()
        .title("This title was edited")
        .description("This description was edited")
        .color(Color::DARK_GREEN)
        .fields(vec![
            ("Edited field 1", "This field was updated", true),
            ("Edited field 2", "This field was also updated", true),
        ])
        .field(
            "New field",
            "This is a new non-inline field added during edit",
            false,
        )
        .footer(footer)
        .timestamp(Timestamp::now());

    println!("Editing message with ID: {}", message_id);

    // Edit the message
    match channel_id
        .edit_message(
            &ctx.http(),
            message_id,
            serenity::builder::EditMessage::new()
                .content("This message was edited!")
                .embed(embed),
        )
        .await
    {
        Ok(_) => {
            println!("Message successfully edited");
            ctx.say("Message successfully edited!").await?;
        }
        Err(why) => {
            let error_msg = format!("Error editing message: {:?}", why);
            println!("{}", error_msg);
            ctx.say(error_msg).await?;
        }
    }

    ctx.say("done.").await?;

    Ok(())
}
