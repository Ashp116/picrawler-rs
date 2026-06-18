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