use crate::debug_log;
use crate::mqtt::types::{
    ControlMessage, MqttConfig, MqttIncoming, PhaseAssignment, PhaseStateMessage,
    PlanStateMessage, TodoStateMessage, WorkspaceRegistryMessage,
};
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, Event, MqttOptions};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

pub struct MqttClient {
    client: AsyncClient,
    config: MqttConfig,
    incoming_tx: broadcast::Sender<MqttIncoming>,
    connected: Arc<Mutex<bool>>,
}

impl MqttClient {
    pub async fn new(config: MqttConfig) -> Self {
        // Unique client ID per process to avoid EMQX kicking duplicate sessions
        let short_id = if config.instance_id.starts_with("weaver-") {
            config.instance_id.trim_start_matches("weaver-").to_string()
        } else {
            config.instance_id.clone()
        };
        let client_id = format!("wvr-{}", short_id);
        eprintln!("[MQTT] Connecting as '{}' to {}:{}", client_id, config.host, config.port);

        let mut mqtt_opts = MqttOptions::new(&client_id, &config.host, config.port);
        mqtt_opts.set_credentials(&config.username, &config.password);
        mqtt_opts.set_keep_alive(Duration::from_secs(60));
        mqtt_opts.set_clean_start(true);
        mqtt_opts.set_max_packet_size(Some(10 * 1024 * 1024));

        let (client, mut eventloop) = AsyncClient::new(mqtt_opts, 64);
        let (incoming_tx, _) = broadcast::channel::<MqttIncoming>(128);

        let connected = Arc::new(Mutex::new(false));
        let connected_clone = connected.clone();
        let tx_clone = incoming_tx.clone();
        let config_clone = config.clone();
        let client_clone = client.clone();

        // Spawn event loop processor on Tauri's async runtime
        tauri::async_runtime::spawn(async move {
            eprintln!("[MQTT] Event loop started, beginning poll cycle");
            let _ = std::fs::write("/tmp/weaver-mqtt-eventloop.log", "eventloop started\n");
            let mut poll_count = 0u64;
            loop {
                poll_count += 1;
                if poll_count <= 5 || poll_count % 100 == 0 {
                    let _ = std::fs::write("/tmp/weaver-mqtt-eventloop.log",
                        format!("poll #{}\n", poll_count));
                }
                match eventloop.poll().await {
                    Ok(event) => {
                        if poll_count <= 10 {
                            let _ = std::fs::write("/tmp/weaver-mqtt-eventloop.log",
                                format!("poll #{}: event={:?}\n", poll_count, event));
                        }
                        match &event {
                            Event::Incoming(Packet::ConnAck(ack)) => {
                                eprintln!("[MQTT] ConnAck received: {:?}", ack);
                                *connected_clone.lock().await = true;
                                debug_log::log_info("[MQTT] Connected to broker");

                                // Subscribe to topics on connect
                                if let Err(e) =
                                    subscribe_topics(&client_clone, &config_clone).await
                                {
                                    debug_log::log_error(&format!(
                                        "[MQTT] Failed to subscribe: {}",
                                        e
                                    ));
                                }
                            }
                            Event::Incoming(Packet::Publish(publish)) => {
                                let topic = std::str::from_utf8(&publish.topic).unwrap_or("");
                                let payload = publish.payload.to_vec();
                                dispatch_message(&tx_clone, &config_clone, topic, &payload);
                            }
                            Event::Incoming(Packet::Disconnect(_)) => {
                                *connected_clone.lock().await = false;
                                debug_log::log_warn("[MQTT] Disconnected from broker");
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        let msg = format!("{}", e);
                        // Only log if not a routine reconnect
                        if !msg.contains("ConnectionAborted") {
                            eprintln!("[MQTT] Connection error: {}", e);
                        }
                        *connected_clone.lock().await = false;
                        debug_log::log_error(&format!("[MQTT] Connection error: {}", e));
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        });

        Self {
            client,
            config,
            incoming_tx,
            connected,
        }
    }

    pub fn subscribe_incoming(&self) -> broadcast::Receiver<MqttIncoming> {
        self.incoming_tx.subscribe()
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    pub async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), String> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.to_vec())
            .await
            .map_err(|e| format!("MQTT publish error: {}", e))
    }

    pub async fn publish_json<T: serde::Serialize>(
        &self,
        topic: &str,
        payload: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_vec(payload).map_err(|e| format!("JSON serialize error: {}", e))?;
        self.publish(topic, &json).await
    }

    /// Publish a JSON payload as a retained message (survives broker restart).
    pub async fn publish_retained<T: serde::Serialize>(
        &self,
        topic: &str,
        payload: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_vec(payload).map_err(|e| format!("JSON serialize error: {}", e))?;
        self.client
            .publish(topic, QoS::AtLeastOnce, true, json)
            .await
            .map_err(|e| format!("MQTT publish retained error: {}", e))
    }

    /// Clear a retained message from the broker by publishing empty payload with retain=true.
    pub async fn clear_retained(&self, topic: &str) -> Result<(), String> {
        self.client
            .publish(topic, QoS::AtLeastOnce, true, vec![])
            .await
            .map_err(|e| format!("MQTT clear retained error: {}", e))
    }

    pub fn config(&self) -> &MqttConfig {
        &self.config
    }
}

async fn subscribe_topics(client: &AsyncClient, config: &MqttConfig) -> Result<(), String> {
    let ws = &config.workspace;
    let iid = &config.instance_id;

    let topics = vec![
        format!("$share/pool/weaver/{}/assign/pool", ws),
        format!("weaver/{}/assign/{}", ws, iid),
        format!("weaver/{}/control/{}", ws, iid),
        format!("weaver/{}/control/all", ws),
        format!("weaver/{}/state/#", ws),
        format!("weaver/{}/registry", ws),
    ];

    for topic in &topics {
        client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| format!("Subscribe error for {}: {}", topic, e))?;
    }

    debug_log::log_info(&format!(
        "[MQTT] Subscribed to {} topics for workspace '{}' instance '{}'",
        topics.len(),
        ws,
        iid
    ));
    Ok(())
}

fn dispatch_message(
    tx: &broadcast::Sender<MqttIncoming>,
    config: &MqttConfig,
    topic: &str,
    payload: &[u8],
) {
    let ws = &config.workspace;
    let parts: Vec<&str> = topic.split('/').collect();

    let msg = if topic == format!("weaver/{}/assign/pool", ws)
        || topic.starts_with(&format!("weaver/{}/assign/", ws))
    {
        // Assignment
        match serde_json::from_slice::<PhaseAssignment>(payload) {
            Ok(assignment) => {
                debug_log::log_info(&format!(
                    "[MQTT] Assignment received: phase={} todos={}",
                    assignment.phase_name,
                    assignment.todos.len()
                ));
                MqttIncoming::Assignment(assignment)
            }
            Err(e) => {
                debug_log::log_error(&format!("[MQTT] Failed to parse assignment: {}", e));
                MqttIncoming::Unknown(topic.to_string(), payload.to_vec())
            }
        }
    } else if topic == format!("weaver/{}/registry", ws) {
        // Workspace registry
        match serde_json::from_slice::<WorkspaceRegistryMessage>(payload) {
            Ok(registry) => {
                debug_log::log_info(&format!(
                    "[MQTT] Registry received: {} missions",
                    registry.missions.len()
                ));
                MqttIncoming::Registry(registry)
            }
            Err(e) => {
                debug_log::log_error(&format!("[MQTT] Failed to parse registry: {}", e));
                MqttIncoming::Unknown(topic.to_string(), payload.to_vec())
            }
        }
    } else if topic.starts_with(&format!("weaver/{}/control/", ws)) {
        // Control
        match serde_json::from_slice::<ControlMessage>(payload) {
            Ok(ctrl) => {
                debug_log::log_info(&format!("[MQTT] Control: action={}", ctrl.action));
                MqttIncoming::Control(ctrl)
            }
            Err(e) => {
                debug_log::log_error(&format!("[MQTT] Failed to parse control: {}", e));
                MqttIncoming::Unknown(topic.to_string(), payload.to_vec())
            }
        }
    } else if parts.len() >= 5 && parts[2] == "state" {
        // State messages: weaver/{ws}/state/{mid}/plan|phase|todo|context
        let kind = if parts.len() >= 5 { parts[4] } else { "" };
        match kind {
            "plan" => match serde_json::from_slice::<PlanStateMessage>(payload) {
                Ok(plan) => MqttIncoming::PlanState(plan),
                Err(_) => MqttIncoming::Unknown(topic.to_string(), payload.to_vec()),
            },
            "phase" if parts.len() >= 6 => {
                match serde_json::from_slice::<PhaseStateMessage>(payload) {
                    Ok(phase) => MqttIncoming::PhaseState(phase),
                    Err(_) => MqttIncoming::Unknown(topic.to_string(), payload.to_vec()),
                }
            }
            "todo" if parts.len() >= 6 => {
                match serde_json::from_slice::<TodoStateMessage>(payload) {
                    Ok(todo) => MqttIncoming::TodoState(todo),
                    Err(_) => MqttIncoming::Unknown(topic.to_string(), payload.to_vec()),
                }
            }
            "context" if parts.len() >= 6 => {
                let todo_id = parts[5].to_string();
                match serde_json::from_slice::<serde_json::Value>(payload) {
                    Ok(bundle) => MqttIncoming::ContextBundle(todo_id, bundle),
                    Err(_) => MqttIncoming::Unknown(topic.to_string(), payload.to_vec()),
                }
            }
            _ => MqttIncoming::Unknown(topic.to_string(), payload.to_vec()),
        }
    } else {
        MqttIncoming::Unknown(topic.to_string(), payload.to_vec())
    };

    let _ = tx.send(msg);
}
