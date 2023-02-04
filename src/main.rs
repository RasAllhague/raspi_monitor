mod handler;
mod sysdata;
mod error;

use std::{
    env,
    sync::atomic::{AtomicBool, AtomicU64}, io,
};

use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};

use tracing::{error, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

#[tokio::main]
#[instrument]
async fn main() {
    dotenv::dotenv().ok();

    let log_directory = env::var("LOG_DIRECTORY").expect("Expected a log directory in the environment.");
    let log_file_prefix = env::var("LOG_FILE_PREFIX").expect("Expected a log file prefix in the environment.");
    let statistics_log_path = env::var("STATISTICS_LOG_PATH").expect("Expected a statistics log file path in the environment.");

    let file_appender = tracing_appender::rolling::hourly(&log_directory, &log_file_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
        .with(fmt::Layer::new().with_writer(io::stdout))
        .with(fmt::Layer::new().with_writer(non_blocking));
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global collector");

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
            statistics_log_path,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
