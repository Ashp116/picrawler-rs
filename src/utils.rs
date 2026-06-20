use std::{sync::{Arc, Mutex}};

use rppal::i2c::I2c;

pub mod adc {
    use std::{fmt::Error};
    use rppal::i2c::{I2c};

    #[derive(Copy, Clone, PartialEq, Debug, Eq)]
    pub enum CHANNEL {
        ADC0,
        ADC1,
        ADC2,
        ADC3,
        ADC4
    }

    impl CHANNEL {
        pub const fn value(self) -> u32 {
            match self {
                CHANNEL::ADC0 => 0x170000,
                CHANNEL::ADC1 => 0x160000,
                CHANNEL::ADC2 => 0x150000,
                CHANNEL::ADC3 => 0x140000,
                CHANNEL::ADC4 => 0x130000,
            }
        }
    }

    pub fn read_raw(channel: CHANNEL) -> Result<u16, Error> {
        let mut i2c = I2c::with_bus(1).unwrap();
        i2c.set_slave_address(0x14).unwrap();
        
        let command = (channel.value() >> 16) as u8;
        i2c.smbus_write_word(command, 0u16).unwrap();

        let mut buf = [0u8; 2];
        i2c.read(&mut buf).unwrap();

        let value = ((buf[0] as u16) << 8 | (buf[1] as u16));
        Ok(value)
    }

    pub fn read_voltage(channel: CHANNEL) -> Result<f32, Error> {
        let value = read_raw(channel).unwrap();
        let voltage = ((value as f32) / 4095_f32) * 3.3;

        Ok(voltage)
    }
}

pub struct  Pwm {
    channel: u8,
    freq: u32,
    bus: Arc<Mutex<I2c>>,

    max_period: u16,
}

impl Pwm {
    const REG_CHN: u8 = 0x20;
    
    const TIMER_PSC: [u8; 3] = [0x40, 0x41, 0x42];
    const TIMER_PER: [u8; 3] = [0x44, 0x45, 0x46];

    const CPU_CLOCK_HZ: f32 = 72_000_000.0;

    fn _i2c_write(&self, reg: u8, value: u16) {
        self.bus.lock().unwrap().smbus_write_word(reg, value.swap_bytes()).unwrap();
    }

    pub fn get_freq(&self) -> u32 {
        self.freq
    }

    pub fn get_period(&self) -> u16 {
        self.max_period
    }
    
    pub fn new(bus: Arc<Mutex<I2c>>, channel: u8, max_period: u16) -> Result<Self, String> {
         if channel > 11 {
            return Err(format!("Channel {} is out of range (0-11)", channel));
        }
        
        let mut pwm = Pwm {
            channel: channel,
            freq: 0,
            bus: bus,
            max_period: max_period,
        };

        pwm.set_freq(50);

        Ok(pwm)
    }
    
    pub fn set_freq(&mut self, hz: u16) {
        let prescaler = (Self::CPU_CLOCK_HZ / (self.max_period as f32 + 1.0) / hz as f32 - 1.0) as u16;
        
        let timer = (self.channel / 4) as usize;

        self._i2c_write(Self::TIMER_PSC[timer], prescaler);
        self._i2c_write(Self::TIMER_PER[timer], self.max_period);
        self.freq = hz as u32;
    }

    pub fn pulse_width(&mut self, pulse_width: u16) {
        let reg: u8 = Self::REG_CHN + self.channel;
        self._i2c_write(reg, pulse_width);
    }
}

pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (value - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}