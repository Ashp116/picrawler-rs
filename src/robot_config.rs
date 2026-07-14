use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RobotConfig {
    pub name: String,
    pub hardware: HardwareConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub geometry: Option<GeometryConfig>,

    pub legs: Vec<LegConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub enable: bool,
    #[serde(default = "default_rate_hz")]
    pub rate_hz: f32,
    #[serde(default)]
    pub socket: TelemetrySocketConfig,
    #[serde(default)]
    pub webui: WebUiConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            enable: false,
            rate_hz: default_rate_hz(),
            socket: TelemetrySocketConfig::default(),
            webui: WebUiConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TelemetrySocketConfig {
    pub bind: String,
    pub port: u16,
}

impl Default for TelemetrySocketConfig {
    fn default() -> Self {
        TelemetrySocketConfig { bind: "0.0.0.0".to_string(), port: 8765 }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebUiConfig {
    pub enable: bool,
    pub bind: String,
    pub port: u16,
    pub root: String,
}

impl Default for WebUiConfig {
    fn default() -> Self {
        WebUiConfig {
            enable: false,
            bind: "0.0.0.0".to_string(),
            port: 8080,
            root: "webui".to_string(),
        }
    }
}

fn default_rate_hz() -> f32 {
    20.0
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct GeometryConfig {
    pub coxa_length_mm: f32,
    pub femur_length_mm: f32,
    pub tibia_length_mm: f32,
}

#[derive(Debug, Deserialize)]
pub struct HardwareConfig {
    pub i2c: I2cConfig,
    pub pwm: PwmConfig,
    pub servos: ServoConfig,
}

#[derive(Debug, Deserialize)]
pub struct I2cConfig {
    pub bus: u8,
    pub robot_hat_address: u8,
}

#[derive(Debug, Deserialize)]
pub struct PwmConfig {
    pub freq_hz: u16,
    pub period: u16,
}

#[derive(Debug, Deserialize)]
pub struct LegConfig {
    pub id: String,
    pub mount_offset_mm: [f32; 3],
    pub joints: Vec<JointConfig>,
}

#[derive(Debug, Deserialize)]
pub struct JointConfig {
    pub name: String,
    pub channel: u8,
    pub calibration_deg: f32,
    pub min_deg: f32,
    pub max_deg: f32,
}

#[derive(Debug, Deserialize)]
pub struct ServoConfig {
    pub zero_on_start: ZeroOnStart,
}

#[derive(Debug, Deserialize)]
pub struct ZeroOnStart {
    pub enable: bool,
    #[serde(default)]
    pub delay: u64,
}