use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
pub async fn bage(
    ctx: Context<'_>,
    #[description = "sjdfsdf user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!(
        "{}'s account doggty dsfdsdfhjwas created at {}",
        u.name,
        u.created_at()
    );
    ctx.say(response).await?;
    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add_player(
    ctx: Context<'_>,
    #[description = "rsn of the player"] user: Option<String>,
    #[description = "team name"] team: Option<String>,
) -> Result<(), Error> {
    if let (Some(user), Some(team)) = (user, team) {
        match add_user_to_team(&user, &team) {
            Ok(_) => {
                let response = format!("{} has been added to team {}", user, team);
                ctx.say(response).await?;
            }
            Err(e) => {
                ctx.say(format!("Failed to add user to team: {}", e))
                    .await?;
                return Ok(());
            }
        }
    } else {
        ctx.say("Please provide a user and team name").await?;
    }

    Ok(())
}

pub fn add_user_to_team(user: &String, team: &String) -> Result<(), Error> {
    // query db
    // add user to team
    // Add the user to the team

    Ok(())
}
