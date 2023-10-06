use crate::consts::BOT_NAME;
use crate::serenity::Colour;
use crate::{Context, Error};

/// Show the help menu. This command can be provided with an option to know more about a command.
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let commands = &ctx.framework().options.commands;

    match command {
        Some(name) => {
            ctx.send(|m| {
                m.embed(|embed| {
                    let c = commands
                        .iter()
                        .find(|&x| x.name == name);

                    match c {
                        Some(x) => embed
                            .colour(Colour::ORANGE)
                            .title(name)
                            .description(x.description.as_deref().unwrap_or("No Description"))
                            .footer(|footer| footer.text(BOT_NAME)),
                        None => embed
                            .colour(Colour::RED)
                            .title("Command not Found")
                            .description("The command you provided does not appear to exist, do /help to see all available commands")
                            .footer(|footer| footer.text(BOT_NAME))
                    }
                })
            })
            .await?;
        }
        None => {
            ctx.send(|m| {
                m.embed(|embed| {
                    embed
                        .colour(Colour::ORANGE)
                        .title("Help")
                        .fields(
                            commands
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    (
                                        c.name.as_str(),
                                        c.description.as_deref().unwrap_or("No Description"),
                                        !((&i + 1) % 3 == 0),
                                    )
                                })
                                .collect::<Vec<(&str, &str, bool)>>(),
                        )
                        .footer(|footer| footer.text(BOT_NAME))
                })
            })
            .await?;
        }
    }

    Ok(())
}
