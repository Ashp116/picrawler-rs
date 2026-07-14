use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use serde::Serialize;

use crate::robot_config::{RobotConfig, WebUiConfig};

/// Static description of the robot from the config, served at /config.json so
/// the dashboard can show real leg names / geometry before any telemetry
/// connection. Shaped like a telemetry snapshot minus the live angles.
#[derive(Serialize)]
struct ConfigPayload<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    name: &'a str,
    geometry: Option<GeometryPayload>,
    legs: Vec<LegPayload<'a>>,
}

#[derive(Serialize)]
struct GeometryPayload {
    coxa_length_mm: f32,
    femur_length_mm: f32,
    tibia_length_mm: f32,
}

#[derive(Serialize)]
struct LegPayload<'a> {
    id: &'a str,
    mount_offset_mm: [f32; 3],
    joints: Vec<JointPayload<'a>>,
}

#[derive(Serialize)]
struct JointPayload<'a> {
    name: &'a str,
    channel: u8,
    min_deg: f32,
    max_deg: f32,
    direction: f32,
}

/// Serialize the robot config into the JSON the dashboard fetches on load.
pub fn config_json(config: &RobotConfig) -> String {
    let payload = ConfigPayload {
        kind: "config",
        name: &config.name,
        geometry: config.geometry.map(|g| GeometryPayload {
            coxa_length_mm: g.coxa_length_mm,
            femur_length_mm: g.femur_length_mm,
            tibia_length_mm: g.tibia_length_mm,
        }),
        legs: config
            .legs
            .iter()
            .map(|l| LegPayload {
                id: &l.id,
                mount_offset_mm: l.mount_offset_mm,
                joints: l
                    .joints
                    .iter()
                    .map(|j| JointPayload {
                        name: &j.name,
                        channel: j.channel,
                        min_deg: j.min_deg,
                        max_deg: j.max_deg,
                        direction: j.direction,
                    })
                    .collect(),
            })
            .collect(),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

/// Best-effort LAN IP of this machine, for printing reachable URLs. Connecting
/// a UDP socket sends no packets; it just resolves the outbound interface.
fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

/// Host to show in printed links: the wildcard bind address isn't reachable,
/// so swap it for the detected LAN IP.
pub fn display_host(bind: &str) -> String {
    if bind == "0.0.0.0" || bind == "::" {
        local_ip().unwrap_or_else(|| "localhost".to_string())
    } else {
        bind.to_string()
    }
}

/// Minimal static file host for the telemetry dashboard. Serves files out of
/// `config.root` (relative to the working directory), `/` maps to index.html,
/// and `/config.json` returns `config_json` (the robot description).
pub fn start(config: &WebUiConfig, config_json: String) -> std::io::Result<()> {
    let addr = format!("{}:{}", config.bind, config.port);
    let listener = TcpListener::bind(&addr)?;
    println!(
        "webui: dashboard ready -> http://{}:{}",
        display_host(&config.bind),
        config.port
    );

    let root = PathBuf::from(&config.root);
    let cfg = Arc::new(config_json);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let root = root.clone();
            let cfg = Arc::clone(&cfg);
            thread::spawn(move || {
                let _ = handle(stream, &root, &cfg);
            });
        }
    });

    Ok(())
}

fn handle(mut stream: TcpStream, root: &Path, config_json: &str) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    // drain the remaining request headers
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let path = path.split('?').next().unwrap_or("/");
    let rel = path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };

    if rel == "config.json" {
        return respond(&mut stream, "200 OK", "application/json", config_json.as_bytes());
    }

    if rel.contains("..") {
        return respond(&mut stream, "404 Not Found", "text/plain", b"not found");
    }

    let file = root.join(rel);
    match fs::read(&file) {
        Ok(body) => respond(&mut stream, "200 OK", content_type(&file), &body),
        Err(_) => respond(&mut stream, "404 Not Found", "text/plain", b"not found"),
    }
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
