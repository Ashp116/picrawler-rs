use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tungstenite::{accept, Message};

use crate::{robot::Robot, robot_config::TelemetryConfig};

#[derive(Serialize)]
struct JointTelemetry {
    name: String,
    channel: u8,
    angle_deg: f32,
    target_deg: f32,
    min_deg: f32,
    max_deg: f32,
    direction: f32,
    at_target: bool,
}

#[derive(Serialize)]
struct LegTelemetry {
    id: String,
    mount_offset_mm: [f32; 3],
    joints: Vec<JointTelemetry>,
}

#[derive(Serialize)]
struct GeometryTelemetry {
    coxa_length_mm: f32,
    femur_length_mm: f32,
    tibia_length_mm: f32,
}

#[derive(Serialize)]
struct Snapshot<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    ts_ms: u128,
    name: &'a str,
    battery_v: Option<f32>,
    all_at_target: bool,
    geometry: Option<GeometryTelemetry>,
    legs: Vec<LegTelemetry>,
}

/// One-way telemetry tunnel: broadcasts the latest robot snapshot as JSON over
/// a websocket to every connected client at the configured rate.
pub struct TelemetryServer {
    latest: Arc<Mutex<String>>,
}

impl TelemetryServer {
    pub fn start(config: &TelemetryConfig) -> std::io::Result<Self> {
        let latest = Arc::new(Mutex::new(
            "{\"type\":\"telemetry\",\"legs\":[]}".to_string(),
        ));

        let addr = format!("{}:{}", config.socket.bind, config.socket.port);
        let listener = TcpListener::bind(&addr)?;
        println!(
            "telemetry: websocket tunnel -> ws://{}:{}",
            crate::webui::display_host(&config.socket.bind),
            config.socket.port
        );

        let interval = Duration::from_millis((1000.0 / config.rate_hz.max(1.0)) as u64);
        let latest_for_accept = Arc::clone(&latest);

        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let latest = Arc::clone(&latest_for_accept);

                thread::spawn(move || {
                    let Ok(mut ws) = accept(stream) else { return };
                    loop {
                        let json = latest.lock().unwrap().clone();
                        if ws.send(Message::Text(json.into())).is_err() {
                            break;
                        }
                        thread::sleep(interval);
                    }
                });
            }
        });

        Ok(TelemetryServer { latest })
    }

    /// Build a snapshot from the robot's current state and stash it for the
    /// client threads to pick up. Cheap enough to call every tick.
    pub fn publish(&self, robot: &Robot, battery_v: Option<f32>) {
        let legs = robot
            .legs
            .iter()
            .zip(robot.config.legs.iter())
            .map(|(leg, leg_cfg)| LegTelemetry {
                id: leg.id.clone(),
                mount_offset_mm: leg_cfg.mount_offset_mm,
                joints: leg
                    .joints
                    .iter()
                    .map(|j| JointTelemetry {
                        name: j.name.clone(),
                        channel: j.channel,
                        angle_deg: robot.get_servo_angle(j.channel).unwrap_or(0.0),
                        target_deg: robot.get_servo_target(j.channel).unwrap_or(0.0),
                        min_deg: j.min_deg,
                        max_deg: j.max_deg,
                        direction: j.direction,
                        at_target: robot.is_servo_at_target(j.channel).unwrap_or(true),
                    })
                    .collect(),
            })
            .collect();

        let snapshot = Snapshot {
            kind: "telemetry",
            ts_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            name: &robot.name,
            battery_v,
            all_at_target: robot.all_servos_at_target(),
            geometry: robot.config.geometry.map(|g| GeometryTelemetry {
                coxa_length_mm: g.coxa_length_mm,
                femur_length_mm: g.femur_length_mm,
                tibia_length_mm: g.tibia_length_mm,
            }),
            legs,
        };

        if let Ok(json) = serde_json::to_string(&snapshot) {
            *self.latest.lock().unwrap() = json;
        }
    }
}
