use std::{error::Error, thread, time::Duration};

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
    CHANNEL: u8,
    FREQ: u32,
    DEVICE: I2c,

    MAX_PERIOD: u16,
}

impl Pwm {
    const REG_CHN: u8 = 0x20;
    const REG_PSC1: u8 = 0x40;
    const REG_PER1: u8 = 0x44;
    const REG_PSC2: u8 = 0x50;
    const REG_PER2: u8 = 0x54;

    const CPU_CLOCK_HZ: f32 = 72_000_000.0;

    fn _i2c_write(&self, reg: u8, value: u16) {
        self.DEVICE.smbus_write_word(reg, value.swap_bytes()).unwrap();
    }
    
    pub fn new(channel: u8, max_period: u16, addr: Option<u16>) -> Result<Self, String> {
         if channel > 11 {
            return Err(format!("Channel {} is out of range (0-11)", channel));
        }

        let mut i2c = I2c::with_bus(1).unwrap();
        i2c.set_slave_address(addr.unwrap_or(0x14)).unwrap();
        
        let mut pwm = Pwm {
            CHANNEL: channel,
            FREQ: 0,
            DEVICE: i2c,
            MAX_PERIOD: max_period,
        };

        pwm.set_freq(50);

        Ok(pwm)
    }

    pub fn set_freq(&mut self, hz: u16) {
        let prescaler = (Self::CPU_CLOCK_HZ / (self.MAX_PERIOD as f32 + 1.0) / hz as f32 - 1.0) as u16;
        
        println!("period: {} | prescaler: {}", self.MAX_PERIOD, prescaler);

        self._i2c_write(Self::REG_PER1, self.MAX_PERIOD);
        self._i2c_write(Self::REG_PSC1, prescaler);
        
        self.FREQ = hz as u32;
    }

    pub fn pulse_width(&mut self, pulse_width: u16) {
        let reg: u8 = Self::REG_CHN + self.CHANNEL;
        self._i2c_write(reg, pulse_width);
    }
}