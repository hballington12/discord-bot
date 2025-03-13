use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

pub fn run(options: &[ResolvedOption]) -> String {
    // extract username and team from options
    "assigning player to team".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("assign")
        .description("Assign a player to a team. Usage: /assign <username> <team>")
}
