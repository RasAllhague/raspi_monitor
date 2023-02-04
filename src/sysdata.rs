use std::io::ErrorKind;

use serde::{Deserialize, Serialize};
use systemstat::{saturating_sub_bytes, Duration, Platform, System};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::error::SysInfoError;

#[derive(Serialize, Deserialize, Clone)]
pub struct SysInfoStrings {
    pub cpu_load: String,
    pub cpu_temp: String,
    pub memory: String,
    pub swap: String,
    pub load_average: String,
    pub uptime: String,
    pub boot_time: String,
    pub socket_stats: String,
}

impl SysInfoStrings {
    pub async fn write_log_entry(&self, log_path: &str) -> Result<(), SysInfoError> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(log_path)
            .await?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let mut sys_infos: Vec<SysInfoStrings> = serde_json::from_str(&contents).unwrap();
        sys_infos.push(self.clone());

        let json = serde_json::to_string(&sys_infos).unwrap();
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }
}

pub async fn get_sysinfo_strings(sys: System) -> SysInfoStrings {
    let memory = get_memory_string(&sys);
    let swap = get_swap_string(&sys);
    let load_average = get_load_avg_string(&sys);
    let uptime = get_uptime_string(&sys);
    let boot_time = get_boot_time_string(&sys);
    let cpu_load = get_cpu_load_string(&sys).await;
    let cpu_temp = get_cpu_temp_string(&sys);
    let socket_stats = get_socket_stats_string(sys);

    SysInfoStrings {
        cpu_load,
        cpu_temp,
        memory,
        swap,
        load_average,
        uptime,
        boot_time,
        socket_stats,
    }
}

fn get_socket_stats_string(sys: System) -> String {
    let socket_stats = match sys.socket_stats() {
        Ok(stats) => format!("{stats:?}"),
        Err(x) => format!("{x}"),
    };
    socket_stats
}

fn get_cpu_temp_string(sys: &System) -> String {
    let cpu_temp = match sys.cpu_temp() {
        Ok(cpu_temp) => format!("{cpu_temp}"),
        Err(x) => format!("{x}"),
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
        Err(x) => format!("\nerror: {x}"),
    };
    cpu_load
}

fn get_boot_time_string(sys: &System) -> String {
    let boot_time = match sys.boot_time() {
        Ok(boot_time) => format!("{boot_time}"),
        Err(x) => format!("error: {x}"),
    };
    boot_time
}

fn get_uptime_string(sys: &System) -> String {
    let uptime = match sys.uptime() {
        Ok(uptime) => format!("{uptime:?}"),
        Err(x) => format!("error: {x}"),
    };
    uptime
}

fn get_load_avg_string(sys: &System) -> String {
    let load_average = match sys.load_average() {
        Ok(loadavg) => format!("{} {} {}", loadavg.one, loadavg.five, loadavg.fifteen),
        Err(x) => format!("error: {x}"),
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
        Err(x) => format!("error: {x}"),
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
        Err(x) => format!("error: {x}"),
    };
    memory
}
