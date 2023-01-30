use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering, AtomicU64},
        Arc,
    },
    time::Duration,
};

use chrono::Utc;
use serenity::{
    async_trait,
    model::prelude::{Activity, ChannelId, Ready, ResumedEvent},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use systemstat::{saturating_sub_bytes, Platform, System};
use tracing::{debug, error, info, instrument};

pub struct Handler {
    pub is_loop_running: AtomicBool,
    pub channel_id: AtomicU64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            let channel_id = self.channel_id.load(Ordering::Relaxed);
            tokio::spawn(async move {
                loop {
                    log_system_load(Arc::clone(&ctx1), channel_id).await;
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

async fn log_system_load(ctx: Arc<Context>, channel_id: u64) {
    let sys = System::new();

    let (memory, swap, load_average, uptime, boot_time, cpu_load, cpu_temp, socket_stats) =
        get_sysinfo_strings(sys).await;

    let message = send_sysinfo_message(
        ctx,
        cpu_load,
        cpu_temp,
        memory,
        swap,
        load_average,
        uptime,
        boot_time,
        socket_stats,
        channel_id
    )
    .await;
    if let Err(why) = message {
        error!("Error sending message: {:?}", why);
    };
}

async fn send_sysinfo_message(
    ctx: Arc<Context>,
    cpu_load: String,
    cpu_temp: String,
    memory: String,
    swap: String,
    load_average: String,
    uptime: String,
    boot_time: String,
    socket_stats: String,
    channel_id: u64,
) -> Result<serenity::model::prelude::Message, serenity::Error> {
    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Resource Load")
                    .field("CPU load", cpu_load, false)
                    .field("CPU temp", cpu_temp, false)
                    .field("Memory", memory, false)
                    .field("Swap", swap, false)
                    .field("Load average", load_average, false)
                    .field("Uptime", uptime, false)
                    .field("Boot time", boot_time, false)
                    .field("System socket statistics", socket_stats, false)
            })
        })
        .await;
    message
}

async fn get_sysinfo_strings(
    sys: System,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let memory = get_memory_string(&sys);
    let swap = get_swap_string(&sys);
    let load_average = get_load_avg_string(&sys);
    let uptime = get_uptime_string(&sys);
    let boot_time = get_boot_time_string(&sys);
    let cpu_load = get_cpu_load_string(&sys).await;
    let cpu_temp = get_cpu_temp_string(&sys);
    let socket_stats = get_socket_stats_string(sys);
    (
        memory,
        swap,
        load_average,
        uptime,
        boot_time,
        cpu_load,
        cpu_temp,
        socket_stats,
    )
}

fn get_socket_stats_string(sys: System) -> String {
    let socket_stats = match sys.socket_stats() {
        Ok(stats) => format!("{:?}", stats),
        Err(x) => format!("{}", x),
    };
    socket_stats
}

fn get_cpu_temp_string(sys: &System) -> String {
    let cpu_temp = match sys.cpu_temp() {
        Ok(cpu_temp) => format!("{}", cpu_temp),
        Err(x) => format!("{}", x),
    };
    cpu_temp
}

async fn get_cpu_load_string(sys: &System) -> String {
    let cpu_load = match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let cpu = cpu.done().unwrap();
            format!(
                "{}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                cpu.user * 100.0,
                cpu.nice * 100.0,
                cpu.system * 100.0,
                cpu.interrupt * 100.0,
                cpu.idle * 100.0
            )
        }
        Err(x) => format!("\nerror: {}", x),
    };
    cpu_load
}

fn get_boot_time_string(sys: &System) -> String {
    let boot_time = match sys.boot_time() {
        Ok(boot_time) => format!("{}", boot_time),
        Err(x) => format!("error: {}", x),
    };
    boot_time
}

fn get_uptime_string(sys: &System) -> String {
    let uptime = match sys.uptime() {
        Ok(uptime) => format!("{:?}", uptime),
        Err(x) => format!("error: {}", x),
    };
    uptime
}

fn get_load_avg_string(sys: &System) -> String {
    let load_average = match sys.load_average() {
        Ok(loadavg) => format!("{} {} {}", loadavg.one, loadavg.five, loadavg.fifteen),
        Err(x) => format!("error: {}", x),
    };
    load_average
}

fn get_swap_string(sys: &System) -> String {
    let swap = match sys.swap() {
        Ok(swap) => format!(
            "{} used / {} ({} bytes) total",
            saturating_sub_bytes(swap.total, swap.free),
            swap.total,
            swap.total.as_u64()
        ),
        Err(x) => format!("error: {}", x),
    };
    swap
}

fn get_memory_string(sys: &System) -> String {
    let memory = match sys.memory() {
        Ok(mem) => format!(
            "{} used / {} ({} bytes) total",
            saturating_sub_bytes(mem.total, mem.free),
            mem.total,
            mem.total.as_u64()
        ),
        Err(x) => format!("error: {}", x),
    };
    memory
}

async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::playing(&formatted_time)).await;
}

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = env::var("RASPI_MONITOR_BOT_TOKEN").expect("Expected a token in the environment.");
    let channel_id = env::var("MONITOR_CHANNEL_ID").expect("Expected channel id in the environment.");

    let intents = GatewayIntents::default();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
            channel_id: AtomicU64::new(channel_id.parse::<u64>().expect("Expected valid channel id in environment.")),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
