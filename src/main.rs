use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::*;
use serenity::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client as Http;

const DISCORD_TOKEN: &str = "PUT_YOUR_TOKEN_HERE";
const AIVEN_API: &str = "https://aiven.onrender.com/chat";

struct Handler {
    http: Arc<Mutex<Http>>,
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Levio online as {}", ready.user.name);

        Command::create_global_application_command(&ctx.http, |c| {
            c.name("ping").description("Check Levio status")
        }).await.unwrap();

        Command::create_global_application_command(&ctx.http, |c| {
            c.name("ask")
                .description("Talk with Levio AI")
                .create_option(|o| {
                    o.name("message")
                        .description("Message")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        }).await.unwrap();

        Command::create_global_application_command(&ctx.http, |c| {
            c.name("register")
                .description("Register your team for tournament")
        }).await.unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            for (_, ch) in channels {
                if ch.name == "welcome" {
                    let _ = ch.send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Welcome to Levelyn Esports")
                                .description(format!(
                                    "Welcome {}, I am **Levio**, your esports assistant.",
                                    member.user.mention()
                                ))
                                .color(0x6f42c1)
                        })
                    }).await;
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {

            if cmd.data.name == "ping" {
                cmd.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| d.content("Levio is fully operational."))
                }).await.unwrap();
            }

            if cmd.data.name == "ask" {
                let q = cmd.data.options[0].value.as_ref().unwrap().as_str().unwrap();
                let reply = ask_ai(q, &self.http).await;

                cmd.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.title("Levio AI")
                                    .description(reply)
                                    .color(0x6f42c1)
                            })
                        })
                }).await.unwrap();
            }

            if cmd.data.name == "register" {
                cmd.create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.embed(|e| {
                                e.title("Team Registration")
                                    .description(
                                        "Please DM Levio with:\n\
                                        • Game Name\n\
                                        • Team Name\n\
                                        • Player List\n\
                                        • Leader Contact\n\
                                        • Tournament Name"
                                    )
                                    .color(0x00ffaa)
                            })
                        })
                }).await.unwrap();
            }
        }
    }
}

async fn ask_ai(prompt: &str, client: &Arc<Mutex<Http>>) -> String {
    let http = client.lock().await;

    let payload = json!({ "message": prompt });

    match http.post(AIVEN_API).json(&payload).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                json["reply"].as_str().unwrap_or("AI error").to_string()
            } else {
                "Invalid AI response".to_string()
            }
        }
        Err(_) => "AI backend unreachable".to_string()
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let handler = Handler {
        http: Arc::new(Mutex::new(Http::new())),
    };

    let mut client = Client::builder(DISCORD_TOKEN, intents)
        .event_handler(handler)
        .await
        .expect("Client error");

    if let Err(e) = client.start().await {
        println!("Client error: {:?}", e);
    }
}
