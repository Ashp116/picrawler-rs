use crate::{actuators::Servo, utils::AtomicF32::AtomicF32};

pub struct ServoGroup {
    servos: Vec<Servo>,
    targets: Vec<AtomicF32>,
}

impl ServoGroup {
    pub fn new() {

    }

    pub fn append(&mut self, mut servo: Servo, disableZero: Option<bool>) {
        if disableZero != Some(true) {
            servo.hard_set_angle(0.0);
        }

        self.targets.insert(servo.channel as usize, AtomicF32::new(0.0));
        self.servos.push(servo);
    }

    pub fn set_target(&mut self, channel: u8, target_angle: f32) {

    }
}