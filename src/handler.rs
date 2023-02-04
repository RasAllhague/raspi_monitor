use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use chrono::Utc;
use serenity::{
    async_trait,
    model::prelude::{Activity, ChannelId, Ready, ResumedEvent},
    prelude::{Context, EventHandler},
};
use systemstat::{Duration, Platform, System};
use tracing::{debug, error, info, instrument};

use crate::sysdata::{get_sysinfo_strings, SysInfoStrings};

pub struct Handler {
    pub is_loop_running: AtomicBool,
    pub channel_id: AtomicU64,
    pub statistics_log_path: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            let channel_id = self.channel_id.load(Ordering::Relaxed);
            let log_path = self.statistics_log_path.clone();
            tokio::spawn(async move {
                loop {
                    log_system_load(Arc::clone(&ctx1), channel_id, &log_path).await;
                    tokio::time::sleep(Duration::from_secs(120)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    set_status_to_current_time(Arc::clone(&ctx2)).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

async fn log_system_load(ctx: Arc<Context>, channel_id: u64, log_path: &str) {
    let sys = System::new();

    let sys_info_strings = get_sysinfo_strings(sys).await;

    if let Err(why) = sys_info_strings.write_log_entry(log_path).await {
        error!("Error writing statistics: {:?}", why);
    };

    let message = send_sysinfo_message(ctx, sys_info_strings, channel_id).await;
    if let Err(why) = message {
        error!("Error sending message: {:?}", why);
    };
}

async fn send_sysinfo_message(
    ctx: Arc<Context>,
    sys_info_strings: SysInfoStrings,
    channel_id: u64,
) -> Result<serenity::model::prelude::Message, serenity::Error> {
    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Resource Load")
                    .field("CPU load", sys_info_strings.cpu_load, false)
                    .field("CPU temp", sys_info_strings.cpu_temp, false)
                    .field("Memory", sys_info_strings.memory, false)
                    .field("Swap", sys_info_strings.swap, false)
                    .field("Load average", sys_info_strings.load_average, false)
                    .field("Uptime", sys_info_strings.uptime, false)
                    .field("Boot time", sys_info_strings.boot_time, false)
                    .field(
                        "System socket statistics",
                        sys_info_strings.socket_stats,
                        false,
                    )
            })
        })
        .await;
    message
}

async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::playing(&formatted_time)).await;
}
