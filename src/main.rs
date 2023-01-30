mod handler;
mod sysdata;

use std::{
    env,
    sync::atomic::{AtomicBool, AtomicU64},
};

use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};

use tracing::{error, instrument};

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = env::var("RASPI_MONITOR_BOT_TOKEN").expect("Expected a token in the environment.");
    let channel_id =
        env::var("MONITOR_CHANNEL_ID").expect("Expected channel id in the environment.");

    let intents = GatewayIntents::default();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
            channel_id: AtomicU64::new(
                channel_id
                    .parse::<u64>()
                    .expect("Expected valid channel id in environment."),
            ),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
