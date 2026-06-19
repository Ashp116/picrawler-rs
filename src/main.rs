use std::{thread, time::Duration};

mod device;
mod utils;

fn main() {
    println!("Hello, world, I am picrawler!");
    device::reset_mcu();
    println!("Reset MCU done!");
    thread::sleep(Duration::from_millis(1000));

    println!("{}v Battery voltage", device::get_battery_voltage().unwrap());
    thread::sleep(Duration::from_secs(5));

    let mut servo = utils::Pwm::new(0, 4095,None).unwrap();
    servo.pulse_width(256);

    thread::sleep(Duration::from_secs(2));
    device::reset_mcu();
    println!("Reset MCU done!");

    // device::reset_mcu();
    // println!("Reset MCU done!");
}
