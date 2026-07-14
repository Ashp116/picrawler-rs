use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
};

use rppal::i2c::I2c;
use crate::actuators::Servo;

pub struct ServoGroup {
    servos: HashMap<u8, Servo>,
    targets: HashMap<u8, f32>,
    estop: Arc<AtomicBool>,
    bus: Arc<Mutex<I2c>>,
}

impl ServoGroup {
    pub fn new(bus: Arc<Mutex<I2c>>) -> Self {
        let group = ServoGroup {
            servos: HashMap::new(),
            targets: HashMap::new(),
            estop: Arc::new(AtomicBool::new(false)),
            bus: Arc::clone(&bus),
        };

        {
            let bus = bus.lock().unwrap();
            for (per_reg, psc_reg) in [(0x44u8, 0x40u8), (0x45, 0x41), (0x46, 0x42)] {
                let _ = bus.smbus_write_word(per_reg, 4095u16.swap_bytes());
                let _ = bus.smbus_write_word(psc_reg, 351u16.swap_bytes());
            }
        }

        group
    }

    pub fn append(&mut self, mut servo: Servo, disable_zero: Option<bool>) {
        let channel = servo.channel;
        if disable_zero != Some(true) {
            servo.soft_set_angle(0.0);
        }
        self.targets.insert(channel, 0.0);
        self.servos.insert(channel, servo);
    }

   pub fn set_target(&mut self, channel: u8, target_angle: f32) {
        if let Some(servo) = self.servos.get_mut(&channel) {
            servo.set_target(target_angle, 90.0);
        }
    }

    pub fn flush(&mut self, dt_ms: f32) {
        let bus = self.bus.lock().unwrap();
        for (channel, servo) in self.servos.iter_mut() {
            let (_angle, pulse_width, _done) = servo.tick(dt_ms);
            let _ = bus.smbus_write_word(0x20 + channel, pulse_width.swap_bytes());
        }
    }

    pub fn tick(&mut self, dt_ms: f32) -> bool {
        if self.estop.load(Ordering::Acquire) {
            return false;
        }
        self.flush(dt_ms);
        true
    }

    pub fn estop(&self) {
        self.estop.store(true, Ordering::SeqCst);
    }

    pub fn get_num_servos(&self) -> usize {
        self.servos.len()
    }
}