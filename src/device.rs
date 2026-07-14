use std::{fmt::Error, thread};
use std::time::Duration;
use rppal::gpio::{Gpio};

use crate::_utils::adc;

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub enum Pins {
    D0,
    D1,
    D2,
    D3,
    Sw,
    User,
    Led,
    BoardType,
    Rst,
    BleInt,
    BleRst,
    McuRst,
    Ce,
}

impl Pins {
    pub const fn value(self) -> u8 {
        match self {
            Pins::D0 => 17,
            Pins::D1 => 4,
            Pins::D2 => 27,
            Pins::D3 => 22,
            Pins::Sw => 25,
            Pins::User => 25,
            Pins::Led => 26,
            Pins::BoardType => 12,
            Pins::Rst => 16,
            Pins::BleInt => 13,
            Pins::BleRst => 20,
            Pins::McuRst => 5,
            Pins::Ce => 8,
        }
    }
}

pub fn set_pin(pin: Pins, high: bool) -> Result<(), rppal::gpio::Error> {
    let gpio = Gpio::new()?;
    
    let mut output_pin = gpio.get(pin.value())?.into_output();
    
    if high {
        output_pin.set_high();
    } else {
        output_pin.set_low();
    }
    
    Ok(())
}

pub fn reset_mcu() {    
    set_pin(Pins::McuRst, false).unwrap();
    thread::sleep(Duration::from_millis(10));
    
    set_pin(Pins::McuRst, true).unwrap();
    thread::sleep(Duration::from_millis(10));
}

pub fn get_battery_voltage() -> Result<f32, Error> {
    let voltage = adc::read_voltage(adc::CHANNEL::ADC4)?;
    Ok(voltage * 3_f32)
}