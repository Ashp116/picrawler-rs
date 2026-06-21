use std::{fs, sync::{Arc, Mutex}, thread, time::Duration};

use rppal::i2c::I2c;

use crate::{actuators::Servo, robot_config::{JointConfig, LegConfig, RobotConfig}};

pub struct Leg {
    pub id: String,
    pub joints: [Joint; 3],
}

impl Leg {
    pub fn from_config(config: &LegConfig, i2c_bus: Arc<Mutex<I2c>>) -> Result<Self, String> {
        let coxa = config.joints.iter().find(|j| j.name == "coxa")
            .ok_or_else(|| format!("leg {}: missing coxa joint", config.id))?;
        let femur = config.joints.iter().find(|j| j.name == "femur")
            .ok_or_else(|| format!("leg {}: missing femur joint", config.id))?;
        let tibia = config.joints.iter().find(|j| j.name == "tibia")
            .ok_or_else(|| format!("leg {}: missing tibia joint", config.id))?;

        Ok(Leg {
            id: config.id.clone(),
            joints: [
                Joint::from_config(coxa, Arc::clone(&i2c_bus)),
                Joint::from_config(femur, Arc::clone(&i2c_bus)),
                Joint::from_config(tibia, Arc::clone(&i2c_bus)),
            ],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub channel: u8,
    pub angle: f32,           
    pub calibration_deg: f32,
    pub min_deg: f32,
    pub max_deg: f32,

    pub servo: Servo,
}

impl Joint {
    pub fn from_config(config: &JointConfig, i2c_bus: Arc<Mutex<I2c>>) -> Self {
        Joint {
            name: config.name.clone(),
            channel: config.channel,
            angle: 0.0,
            calibration_deg: config.calibration_deg,
            min_deg: config.min_deg,
            max_deg: config.max_deg,

            servo: Servo::new(Arc::clone(&i2c_bus), config.channel, None, None).unwrap(),
        }
    }
}

pub struct Robot {
    pub name: String,
    pub i2c_bus: Arc<Mutex<I2c>>,
    pub legs: Vec<Leg>,
}

impl Robot {
    pub fn from_config(config: RobotConfig) -> Result<Self, String> {
        let mut i2c = I2c::with_bus(config.hardware.i2c.bus).map_err(|e| e.to_string())?;
        i2c.set_slave_address(config.hardware.i2c.robot_hat_address as u16)
            .map_err(|e| e.to_string())?;

        let i2c = Arc::new(Mutex::new(i2c));

        let legs: Result<Vec<Leg>, String> = config.legs.iter()
            .map(|l| Leg::from_config(l, Arc::clone(&i2c)))
            .collect();

        Ok(Robot {
            name: config.name,
            i2c_bus: i2c,
            legs: legs.unwrap(),
        })
    }

    pub fn from_yaml(robot_yaml: String) -> Result<Self, String> {
        let content = fs::read_to_string(robot_yaml).map_err(|e| e.to_string()).unwrap();
        let config = serde_yaml::from_str::<RobotConfig>(&content).map_err(|e| e.to_string()).unwrap();

        Self::from_config(config)
    }


    pub fn set_servo_angle(&mut self, angle: f32) {
        self.legs.iter_mut().for_each(|leg| {
            leg.joints.iter_mut().for_each(|joint| {
                joint.servo.set_angle(angle.clamp(joint.min_deg, joint.max_deg));
            });
            thread::sleep(Duration::from_millis(70));
        });
    }
    
}

