use crate::mqtt::client::MqttClient;
use crate::mqtt::state_cache::MissionStateCache;
use crate::mqtt::types::HeartbeatEvent;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub fn start_heartbeat(
    mqtt: Arc<Mutex<Option<MqttClient>>>,
    cache: Arc<Mutex<MissionStateCache>>,
    instance_id: String,
    workspace: String,
    capacity: u32,
    app_start: Instant,
) {
    let hostname = gethostname::gethostname()
        .to_string_lossy()
        .to_string();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            let guard = mqtt.lock().await;
            let client = match guard.as_ref() {
                Some(c) if c.is_connected().await => c,
                _ => continue,
            };

            // Count active Claude Code sessions via sysinfo
            let active_agents = count_claude_processes();

            // Get claimed mission IDs from cache
            let claimed_missions = {
                let c = cache.lock().await;
                c.get_claimed_mission_ids()
            };

            let event = HeartbeatEvent {
                instance_id: instance_id.clone(),
                workspace: workspace.clone(),
                hostname: hostname.clone(),
                capacity,
                active_agents,
                missions: claimed_missions,
                instance_type: "local".into(),
                tags: HashMap::from([
                    ("os".into(), std::env::consts::OS.into()),
                    ("arch".into(), std::env::consts::ARCH.into()),
                ]),
                uptime_seconds: app_start.elapsed().as_secs(),
                published_at: Utc::now().to_rfc3339(),
            };

            // Publish as retained so Brain/Obsidian always has the latest device info
            let topic = format!("brain/{}/heartbeat/{}", workspace, instance_id);
            if let Err(e) = client.publish_retained(&topic, &event).await {
                crate::debug_log::log_error(&format!("[MQTT] Heartbeat publish error: {}", e));
            }
        }
    });
}

fn count_claude_processes() -> u32 {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cmd(UpdateKind::OnlyIfNotSet),
    );

    sys.processes()
        .values()
        .filter(|p| {
            let name = p.name().to_string_lossy().to_lowercase();
            name == "claude" || name.starts_with("claude-")
        })
        .count() as u32
}
