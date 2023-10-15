mod commands;
mod consts;

use axum::{
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    routing::{get, post},
    Json, Router, TypedHeader,
};

use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, Activity, CacheAndHttp, ChannelId, GuildId};
use serde::Deserialize;
use std::{env::var, sync::Arc, time::Duration};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            eprintln!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                eprintln!("Error while handling error: {}", e)
            }
        }
    }
}

#[derive(Deserialize)]
struct ReqData {
    user: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![commands::help(), commands::update()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("--".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            additional_prefixes: vec![poise::Prefix::Literal(consts::MENTION)],
            ..Default::default()
        },
        /// The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),

        /// This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!(
                    "\x1b[1m\x1b[35mCommand\x1b[0m {} being executed...",
                    ctx.command().qualified_name
                );
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!(
                    "\x1b[1m\x1b[35mCommand\x1b[0m {} executed!",
                    ctx.command().qualified_name
                );
            })
        },
        /// Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),

        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("\x1b[1m\x1b[34mEvent\x1b[0m in handler {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .token(var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var"))
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                println!("\x1b[1m\x1b[32mReady\x1b[0m as {}", ready.user.tag());

                ctx.set_activity(Activity::listening(format!("to -- prefix")))
                    .await;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .build();

    let fwork = framework.await.unwrap();
    let http = fwork.client().cache_and_http.clone();

    tokio::task::spawn(async move {
        let app = Router::new()
            .route("/", get(|| async { "Bot hosting running correctly!" }))
            .route("/vote", post(move |auth, data| send_vote(auth, data, http)));

        axum::Server::bind(&"0.0.0.0:3000".parse().expect("Failed to parse host"))
            .serve(app.into_make_service())
            .await
            .expect("Failed to start server");
    });

    fwork.start().await.expect("Failed to start bot");
}

async fn send_vote(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(data): Json<ReqData>,
    http: Arc<CacheAndHttp>,
) -> (StatusCode, String) {
    if auth.token() != var("TOPGG_AUTH").unwrap_or(String::new()) {
        (
            StatusCode::UNAUTHORIZED,
            format!("You're not authorized to do this operation"),
        )
    } else {
        ChannelId(1097551957570375750).send_message(&http.http, |m| {
            m.embed(|embed| {
                embed
                    .title("<:pyrite:1112834249385578517> New Vote!")
                    .description(format!("<:arrow:1068604670764916876> <@{}> Just Voted **__Pyrite Bot__** on [Top.gg](https://top.gg/bot/1008400801628164096)! They have now received the <@&1024001432078258216> Role! You can vote too to get it!\n\n:heart: __Thank you for voting Pyrite Bot__", data.user))
                    .colour(0x2b2d31)
                    .thumbnail("https://i.imgur.com/bbH7fEf.png")
                    .footer(|footer| footer.text("Pyrite Bot Support").icon_url("https://i.imgur.com/bbH7fEf.png"))
            })
        }).await.expect("There was an error sending the vote!");

        GuildId(1008365644636495953)
            .member(
                &http.http,
                data.user.parse::<u64>().unwrap_or(807705107852558386),
            )
            .await
            .expect("Member not found")
            .add_role(&http.http, 1024001432078258216)
            .await
            .expect("There was an error adding the role");

        println!("New Vote from {}", data.user);

        (StatusCode::OK, String::new())
    }
}
