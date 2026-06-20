use std::{sync::{Arc, Mutex}};

use rppal::i2c::I2c;

use crate::utils::{Pwm, map_range};

#[derive(Debug, Clone)]
pub struct Servo {
    pwm: Pwm,
    max_pw: u32,
    min_pw: u32,
}

impl Servo {
    const MAX_PW: u32 = 2500;
    const MIN_PW: u32 = 500;
    const DEFAULT_FREQ: u16 = 50;
    const MAX_PREIOD: u16 = 4095;

    pub fn new(bus: Arc<Mutex<I2c>>, pin: u8, max_period: Option<u16>, freq: Option<u16>) -> Result<Self, String> {
        let mut pwm = Pwm::new(bus, pin as u8, max_period.unwrap_or(Self::MAX_PREIOD)).unwrap();
        
        if freq != None {
            //pwm.set_freq(freq.unwrap_or(Self::DEFAULT_FREQ));
        }
        
        Ok(Servo { 
            pwm, 
            max_pw: Self::MAX_PW, 
            min_pw: Self::MIN_PW 
        })
    }

    pub fn set_min_max_pw(&mut self, min_pw: u32, max_pw: u32) {
        self.min_pw = min_pw;
        self.max_pw = max_pw;
    }
    
    pub fn set_angle(&mut self, mut angle: i16) {
        angle = angle.clamp(-90, 90);

        let pwm_width_time = map_range(angle as f32, -90f32, 90f32, self.min_pw as f32, self.max_pw as f32);
        self.set_pluse_width_time(pwm_width_time);
    }

    pub fn set_pluse_width_time(&mut self, mut pluse_width_time: f32) {
        pluse_width_time = pluse_width_time.clamp(self.min_pw as f32, self.max_pw as f32);
        
        let period_us = 1_000_000.0 / self.pwm.get_freq() as f32;
        let pwr = pluse_width_time / period_us;
        let pulse_width = (pwr * self.pwm.get_period() as f32) as u16;

        self.pwm.pulse_width(pulse_width);
    }
} 