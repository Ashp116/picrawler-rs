use std::{fmt::Error, thread};
use std::time::Duration;
use rppal::gpio::{Gpio};

use crate::utils::adc;

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub enum pins {
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

impl pins {
    pub const fn value(self) -> u8 {
        match self {
            pins::D0 => 17,
            pins::D1 => 4,
            pins::D2 => 27,
            pins::D3 => 22,
            pins::Sw => 25,
            pins::User => 25,
            pins::Led => 26,
            pins::BoardType => 12,
            pins::Rst => 16,
            pins::BleInt => 13,
            pins::BleRst => 20,
            pins::McuRst => 5,
            pins::Ce => 8,
        }
    }
}

pub fn set_pin(pin: pins, high: bool) -> Result<(), rppal::gpio::Error> {
    let gpio = Gpio::new()?;
    let mut output_pin = gpio.get(pin.value()).expect("Pin not found!").into_output();

    if high {
        output_pin.set_high();
    }
    else {
        output_pin.set_low();
    }

    thread::sleep(Duration::from_millis(1));
    Ok(())
}

pub fn reset_mcu() {
    set_pin(pins::McuRst, false).unwrap();
    thread::sleep(Duration::from_millis(1));
    set_pin(pins::McuRst, false).unwrap();
    thread::sleep(Duration::from_millis(1));
}

pub fn get_battery_voltage() -> Result<f32, Error> {
    let voltage = adc::read_voltage(adc::CHANNEL::ADC4).unwrap();
    Ok(voltage * 3_f32)
}