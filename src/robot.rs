use std::{fs, sync::{Arc, Mutex}, thread, time::Duration};

use rppal::i2c::I2c;

use crate::{actuator_group::ServoGroup, actuators::Servo, robot_config::{LegConfig, JointConfig, RobotConfig}};

pub struct Leg {
    pub id: String,
    pub joints: [Joint; 3],
}

impl Leg {
    pub fn from_config(config: &LegConfig) -> Result<Self, String> {
        let coxa = config.joints.iter().find(|j| j.name == "coxa")
            .ok_or_else(|| format!("leg {}: missing coxa joint", config.id))?;
        let femur = config.joints.iter().find(|j| j.name == "femur")
            .ok_or_else(|| format!("leg {}: missing femur joint", config.id))?;
        let tibia = config.joints.iter().find(|j| j.name == "tibia")
            .ok_or_else(|| format!("leg {}: missing tibia joint", config.id))?;

        Ok(Leg {
            id: config.id.clone(),
            joints: [
                Joint::from_config(coxa),
                Joint::from_config(femur),
                Joint::from_config(tibia),
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
}

impl Joint {
    pub fn from_config(config: &JointConfig) -> Self {
        Joint {
            name: config.name.clone(),
            channel: config.channel,
            angle: 0.0,
            calibration_deg: config.calibration_deg,
            min_deg: config.min_deg,
            max_deg: config.max_deg,
        }
    }
}

pub struct Robot {
    pub name: String,
    pub i2c_bus: Arc<Mutex<I2c>>,
    pub legs: Vec<Leg>,
    pub config: RobotConfig,
    servo_group: ServoGroup,
}

impl Robot {
    pub fn from_config(config: RobotConfig) -> Result<Self, String> {
        let mut i2c = I2c::with_bus(config.hardware.i2c.bus).map_err(|e| e.to_string())?;
        i2c.set_slave_address(config.hardware.i2c.robot_hat_address as u16)
            .map_err(|e| e.to_string())?;

        let i2c = Arc::new(Mutex::new(i2c));

        let legs: Vec<Leg> = config.legs.iter()
            .map(|l| Leg::from_config(l))
            .collect::<Result<Vec<Leg>, String>>()?;

        let mut servo_group = ServoGroup::new(Arc::clone(&i2c));

        for leg in legs.iter() {
            for joint in leg.joints.iter() {
                let servo = Servo::new(
                    joint.channel,
                    0.0,
                    joint.min_deg,
                    joint.max_deg,
                    joint.calibration_deg,
                );

                servo_group.append(servo, Some(false));
            }
        }

        if config.hardware.servos.zero_on_start.enable {
            servo_group.flush(0.0);
            thread::sleep(Duration::from_millis(config.hardware.servos.zero_on_start.delay));
        }

        Ok(Robot {
            name: config.name.clone(),
            i2c_bus: Arc::clone(&i2c),
            legs,
            config,
            servo_group,
        })
    }

    pub fn from_yaml(robot_yaml: String) -> Result<Self, String> {
        let content = fs::read_to_string(robot_yaml).map_err(|e| e.to_string())?;
        let config = serde_yaml::from_str::<RobotConfig>(&content).map_err(|e| e.to_string())?;
        Self::from_config(config)
    }

    pub fn set_servo_angle(&mut self, angle: f32) {
        for leg in self.legs.iter() {
            for joint in leg.joints.iter() {
                self.servo_group.set_target(joint.channel, angle);
            }
        }
    }

    pub fn tick(&mut self, dt_ms: f32) {
        self.servo_group.tick(dt_ms);
    }
}