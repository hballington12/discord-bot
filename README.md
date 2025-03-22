# discord-bot

Notes:

- need to disable dink rich embeds in advanced section of plugin    //// TEMPORARY DISABLE
    // // Check if team has access to monsters of this combat level
    // if !get_team_armory_level(pool, source_combat_level, team.0)
    //     .await?
    //     .unwrap_or(false)
    // {
    //     println!(
    //         "Team '{}' doesn't have access to combat level {} monsters",
    //         team.1, source_combat_level
    //     );
    //     return Ok(());
    // }

    match data.bestiary.get_slayer_level(&drop.source) {
        Some(level) => {
            println!("Slayer level for source '{}': {}", drop.source, level);

            // Check if team has necessary slayer level
            if !get_team_slayer_level(pool, level as i32, team.0)
                .await?
                .unwrap_or(false)
            {
                println!(
                    "Team '{}' doesn't have access to slayer level {} monsters",
                    team.1, level
                );
                return Ok(());
            }
        }
        None => {}
    }