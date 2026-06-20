use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RobotConfig {
    pub name: String,
    pub hardware: HardwareConfig,
}

#[derive(Debug, Deserialize)]
pub struct HardwareConfig {
    pub i2c: I2cConfig,
    pub pwm: PwmConfig,
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
    pub joints: Vec<JointConfig>,  // was HashMap<String, JointConfig>
}

#[derive(Debug, Deserialize)]
pub struct JointConfig {
    pub name: String,
    pub channel: u8,
    pub calibration_deg: f32,
    pub min_deg: f32,
    pub max_deg: f32,
}