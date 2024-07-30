use env_logger::{Builder, Target};
use std::env;
use std::fs;

use ollama_rs::generation::options::GenerationOptions;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod model;
use config::Config;
use model::Model;

#[allow(dead_code)]
struct DiscordBot {
    model: Arc<Mutex<Model>>,
}

impl DiscordBot {
    fn new(model: Model) -> Self {
        Self {
            model: Arc::new(Mutex::new(model)),
        }
    }
}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let bot_id = ctx.cache.current_user().id;

        let user = msg.author.name;
        log::info!("Received message from: {}", user);
        log::info!("Embeds: {:?}", msg.embeds);
        log::info!("Content: {:?}", msg.content);

        // If direct message
        if msg.guild_id.is_none() {
            let mut model = self.model.lock().await;
            let reply = model.send_message(&user, &msg.content).await;
            if let Some(reply) = reply {
                if let Err(why) = msg.channel_id.say(&ctx.http, &reply).await {
                    log::error!("{why:?}");
                    return;
                }
                log::info!("{reply}");
            }
        }

        // If bot mentioned
        if msg.mentions.iter().any(|user| user.id == bot_id) {
            let mut model = self.model.lock().await;
            let reply = model.send_message(&user, &msg.content).await;
            if let Some(reply) = reply {
                if let Err(why) = msg.channel_id.say(&ctx.http, &reply).await {
                    log::error!("{why:?}");
                    return;
                }
                log::info!("{reply}");
            }
        }
    }
}

fn parse_config() -> Option<Config> {
    let cwd = env::current_dir().ok()?;
    let config_path = cwd.join("config.yaml");
    if config_path.exists() {
        if let Ok(file_contents) = fs::read_to_string(config_path) {
            serde_yaml::from_str(&file_contents).ok()
        } else {
            None
        }
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    // Initialize the logger
    Builder::new()
        .filter_module(module_path!(), log::LevelFilter::Debug)
        .target(Target::Stdout)
        .init();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let model;
    let token;

    if let Some(config) = parse_config() {
        // create model
        model = Model::new(
            config.model_name,
            GenerationOptions::default()
                .mirostat(config.mirostat)
                .num_ctx(config.context_size),
            config.system_prompt,
        );

        token = config.bot_token;
    } else {
        log::error!("Failed to parse config file");
        return;
    }

    log::info!("Starting client...");

    // Create a new instance of the Client, logging in as a bot.
    let res = Client::builder(&token, intents)
        .event_handler(DiscordBot::new(model))
        .await;

    match res {
        Ok(mut client) => {
            log::info!("Client created!");
            // Start listening for events by starting a single shard
            if let Err(why) = client.start().await {
                log::error!("{why:?}");
            }

            log::info!("Client created!");
        }
        Err(why) => {
            log::error!("{why:?}");
        }
    }
}
