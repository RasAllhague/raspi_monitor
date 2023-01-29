# raspi_monitor

Discord bot for monitoring raspberry pis and other systems.
Sends a message every 1 minute.

Uses the systemstat crate.

## Tested on:
 - Raspberry Pi 4B (Raspberry Pi OS)
 - Manjaro

## Features:
 - show cpu load
 - show cpu temperatur
 - show memory usage
 - show swap usage
 - show load average
 - show uptime
 - show boot time
 - system socket statistics

## Environment variables:
 - RASPI_MONITOR_BOT_TOKEN
 - MONITOR_CHANNEL_ID