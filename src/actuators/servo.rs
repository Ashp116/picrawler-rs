use std::{cmp::{max, min}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use rppal::i2c::I2c;

use crate::utils::{Pwm, map_range};

#[derive(Debug, Clone)]
pub struct Servo {
    pwm: Pwm,
    max_pw: u32,
    min_pw: u32,
    current_angle: f32,
}

impl Servo {
    const MAX_PW: u32 = 2500;
    const MIN_PW: u32 = 500;
    const DEFAULT_FREQ: u16 = 50;
    const MAX_PREIOD: u16 = 4095;

    fn _step_angle(&mut self, target_angle: f32, speed: Option<f32>) {
        let speed = speed.unwrap_or(100.0).clamp(0.0, 100.0);
        let step_time_ms: u64 = 10;

        let total_time = -9.9 * speed + 1000.0;
        let delta = target_angle - self.current_angle;

        if delta.abs() < 0.01 {
            return;
        }

        let max_dps = 428.0; 
        let current_dps = delta.abs() / total_time * 1000.0;
        let total_time = if current_dps > max_dps {
            delta.abs() / max_dps * 1000.0
        } else {
            total_time
        };

        let max_step = (total_time / step_time_ms as f32) as u32;
        let max_step = max_step.max(1);
        let step = delta / max_step as f32; 

        for _ in 0..max_step {
            let tick_start = Instant::now();

            self.current_angle += step;
            let pwm_width_time = map_range(
                self.current_angle,
                -90.0, 90.0,
                self.min_pw as f32, self.max_pw as f32
            );
            self.set_pulse_width_time(pwm_width_time);

            let elapsed = tick_start.elapsed();
            let step_dur = Duration::from_millis(step_time_ms);
            if elapsed < step_dur {
                thread::sleep(step_dur - elapsed);
            }
        }

        self.current_angle = target_angle;
        let final_pw = map_range(target_angle, -90.0, 90.0, self.min_pw as f32, self.max_pw as f32);
        self.set_pulse_width_time(final_pw);
    }

    pub fn new(bus: Arc<Mutex<I2c>>, pin: u8, init_angle: f32, max_period: Option<u16>, freq: Option<u16>) -> Result<Self, String> {
        let mut pwm = Pwm::new(bus, pin as u8, max_period.unwrap_or(Self::MAX_PREIOD)).unwrap();
        
        if freq != None {
            //pwm.set_freq(freq.unwrap_or(Self::DEFAULT_FREQ));
        }
        
        Ok(Servo { 
            pwm, 
            max_pw: Self::MAX_PW, 
            min_pw: Self::MIN_PW,
            current_angle: init_angle,
        })
    }

    pub fn set_min_max_pw(&mut self, min_pw: u32, max_pw: u32) {
        self.min_pw = min_pw;
        self.max_pw = max_pw;
    }

    pub fn soft_set_angle(&mut self, angle: f32) {
        self.current_angle = angle
    }

    pub fn hard_set_angle(&mut self, mut angle: f32) {
        angle = angle.clamp(-90.0, 90.0);

        let final_pw = map_range(angle, -90.0, 90.0, self.min_pw as f32, self.max_pw as f32);
        self.set_pulse_width_time(final_pw);
    }
    
    pub fn set_angle(&mut self, mut angle: f32) {
        angle = angle.clamp(-90.0, 90.0);

        self._step_angle(angle, None);
    }

    pub fn set_pulse_width_time(&mut self, mut pluse_width_time: f32) {
        pluse_width_time = pluse_width_time.clamp(self.min_pw as f32, self.max_pw as f32);
        
        let period_us = 1_000_000.0 / self.pwm.get_freq() as f32;
        let pwr = pluse_width_time / period_us;
        let pulse_width = (pwr * self.pwm.get_period() as f32) as u16;

        self.pwm.pulse_width(pulse_width);
    }
} 
