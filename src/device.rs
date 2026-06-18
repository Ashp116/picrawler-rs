use std::thread;
use std::time::Duration;
use rppal::gpio::{Error, Gpio};

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub enum PINS {
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

impl PINS {
    pub const fn value(self) -> u8 {
        match self {
            PINS::D0 => 17,
            PINS::D1 => 4,
            PINS::D2 => 27,
            PINS::D3 => 22,
            PINS::Sw => 25,
            PINS::User => 25,
            PINS::Led => 26,
            PINS::BoardType => 12,
            PINS::Rst => 16,
            PINS::BleInt => 13,
            PINS::BleRst => 20,
            PINS::McuRst => 5,
            PINS::Ce => 8,
        }
    }
}

pub fn set_pin(pin: PINS, high: bool) -> Result<(), rppal::gpio::Error> {
    println!("{} is the value", pin.value());

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
    set_pin(PINS::McuRst, false).unwrap();
    thread::sleep(Duration::from_millis(1));
    set_pin(PINS::McuRst, false).unwrap();
    thread::sleep(Duration::from_millis(1));
}