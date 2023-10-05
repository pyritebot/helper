use crate::{Data, Error};
use poise::{
    serenity_prelude::{CacheHttp, ChannelId},
    Modal,
};

type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;

#[derive(Modal)]
#[name = "Add an update!"]
struct UpdateModal {
    #[name = "Version"]
    #[placeholder = "ex: 1.0.0"]
    #[min_length = 5]
    #[max_length = 5]
    version: String,
    #[name = "Description"]
    #[placeholder = "Version Changes"]
    #[paragraph]
    description: String,
}

/// Add an update!
#[poise::command(slash_command)]
pub async fn update(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = UpdateModal::execute(ctx).await?;

    match data {
        Some(modal) => {
            ChannelId(1023607934825013310)
                .send_message(&ctx.serenity_context.http(), |m| {
                    m.content(format!(
                        "<:arrow:1068604670764916876> **Pyrite v{} Update Log**\n   ・ {}",
                        modal.version,
                        modal.description.replace("\n", "\n   ・ ")
                    ))
                })
                .await?;
        }
        None => {
            ctx.send(|m| {
                m.ephemeral(true)
                    .content("There was an error loading the data!")
            })
            .await?;
        }
    }

    Ok(())
}
