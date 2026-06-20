use std::{fs, sync::{Arc, Mutex}};

use rppal::i2c::I2c;

use crate::robot_config::RobotConfig;

pub struct Robot {
    pub name: String,
    pub i2c_bus: Arc<Mutex<I2c>>,
}

impl Robot {
    pub fn from_config(config: RobotConfig) -> Result<Self, String> {
        let mut i2c = I2c::with_bus(config.hardware.i2c.bus).map_err(|e| e.to_string())?;
        i2c.set_slave_address(config.hardware.i2c.robot_hat_address as u16)
            .map_err(|e| e.to_string())?;

        Ok(Robot {
            name: config.name,
            i2c_bus: Arc::new(Mutex::new(i2c)),
        })
    }

    pub fn from_yaml(robot_yaml: String) -> Result<Self, String> {
        let content = fs::read_to_string(robot_yaml).map_err(|e| e.to_string()).unwrap();
        let config = serde_yaml::from_str::<RobotConfig>(&content).map_err(|e| e.to_string()).unwrap();

        Self::from_config(config)
    }

    
}

