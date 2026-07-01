use std::{collections::HashMap, num, sync::{Arc, Barrier, atomic::{AtomicBool, Ordering}}, thread};

use crate::{actuators::Servo, utils::AtomicF32::AtomicF32};

pub struct ServoGroup {
    targets: HashMap<u8, Arc<AtomicF32>>,
    estop: Arc<AtomicBool>,

    num_servos: usize,
    heartbeat: Arc<Barrier>,
}

impl ServoGroup {
    pub fn new(num_servos: usize) -> Self {
        return ServoGroup { 
            targets: HashMap::new(), 
            estop: Arc::new(AtomicBool::new(false)),
            num_servos: num_servos as usize,
            heartbeat: Arc::new(Barrier::new((num_servos + 1) as usize))
        }
    }

    pub fn append(&mut self, mut servo: Servo, disableZero: Option<bool>) {
        if disableZero != Some(true) {
            servo.hard_set_angle(0.0);
        }

        let estop = Arc::clone(&self.estop);
        let target = Arc::new(AtomicF32::new(0.0));
        let target_clone = Arc::clone(&target);
        let heartbeat = Arc::clone(&self.heartbeat);

        self.targets.insert(servo.channel, target);

        thread::spawn(move || {
            loop {
                heartbeat.wait();
                if estop.load(Ordering::Acquire) { break; }

                let target_angle = target_clone.load();
                servo.set_angle(target_angle);
                println!("{} servo, {} angle", servo.channel, target_angle);
            }
        });
    }

    pub fn set_target(&mut self, channel: u8, target_angle: f32) {
        self.targets[&channel].store(target_angle);
    }

    pub fn estop(&self) {
        self.estop.store(true, Ordering::SeqCst);
    }

    pub fn tick(&self) -> bool {
        if self.estop.load(Ordering::Acquire) {
            return false; // don't wait, just return
        }
        self.heartbeat.wait();
        true
    }
}