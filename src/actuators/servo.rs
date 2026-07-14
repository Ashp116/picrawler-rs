use crate::_utils::map_range;

#[derive(Debug, Clone)]
pub struct Servo {
    pub channel: u8,
    pub max_pw: f32,
    pub min_pw: f32,
    pub current_angle: f32,
    pub min_deg: f32,
    pub max_deg: f32,
    pub calibration_deg: f32,
    velocity: f32,
    target: f32,
}

impl Servo {
    const MAX_PW: f32 = 2500.0;
    const MIN_PW: f32 = 500.0;
    const MAX_DPS: f32 = 428.0;

    pub fn new(channel: u8, init_angle: f32, min_deg: f32, max_deg: f32, calibration_deg: f32) -> Self {
        Servo {
            channel,
            max_pw: Self::MAX_PW,
            min_pw: Self::MIN_PW,
            current_angle: init_angle,
            min_deg,
            max_deg,
            calibration_deg,
            velocity: 0.0,
            target: init_angle,
        }
    }

    pub fn angle_to_pw(&self, angle: f32) -> u16 {
        let angle = angle.clamp(self.min_deg, self.max_deg);
        let pw_time = map_range(angle + self.calibration_deg, -90.0, 90.0, self.min_pw, self.max_pw);
        let period_us = 1_000_000.0 / 50.0;
        ((pw_time / period_us) * 4095.0) as u16
    }

    pub fn soft_set_angle(&mut self, angle: f32) {
        self.current_angle = angle.clamp(self.min_deg, self.max_deg);
        self.target = self.current_angle;
        self.velocity = 0.0;
    }

    pub fn set_target(&mut self, target: f32, speed: f32) {
        let target = target.clamp(self.min_deg, self.max_deg);
        let delta = target - self.current_angle;

        if delta.abs() < 0.01 {
            self.velocity = 0.0;
            self.target = target;
            return;
        }

        let speed = speed.clamp(0.0, 100.0);
        let mut total_time_ms = -9.9 * speed + 1000.0;
        let current_dps = delta.abs() / total_time_ms * 1000.0;
        if current_dps > Self::MAX_DPS {
            total_time_ms = delta.abs() / Self::MAX_DPS * 1000.0;
        }

        self.target = target;
        self.velocity = delta / total_time_ms;
    }

    /// True once the servo has finished stepping and is holding its goal position.
    pub fn is_at_target(&self) -> bool {
        self.velocity == 0.0
    }

    pub fn get_target(&self) -> f32 {
        self.target
    }

    /// Advances the servo by `dt_ms` of real elapsed time. Returns (angle, pulse_width, done).
    pub fn tick(&mut self, dt_ms: f32) -> (f32, u16, bool) {
        if self.velocity == 0.0 {
            return (self.current_angle, self.angle_to_pw(self.current_angle), true);
        }

        let next = self.current_angle + self.velocity * dt_ms;
        let overshot = if self.velocity > 0.0 { next >= self.target } else { next <= self.target };

        if overshot {
            self.current_angle = self.target;
            self.velocity = 0.0;
        } else {
            self.current_angle = next;
        }

        let pw = self.angle_to_pw(self.current_angle);
        (self.current_angle, pw, self.velocity == 0.0)
    }
}

impl Default for Servo {
    fn default() -> Self {
        Servo::new(0, 0.0, -90.0, 90.0, 0.0)
    }
}