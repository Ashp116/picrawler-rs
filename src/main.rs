use std::{thread, time::Duration};

use crate::servo::Servo;

mod device;
mod utils;
mod servo;

fn main() {
    println!("Hello, world, I am picrawler!");
    device::reset_mcu();
    println!("Reset MCU done!");
    thread::sleep(Duration::from_millis(1000));

    println!("{}v Battery voltage", device::get_battery_voltage().unwrap());
    thread::sleep(Duration::from_secs(1));

    // let mut servo0 = Servo::new(0, None, None, None).unwrap();
    // let mut servo1 = Servo::new(1, None, None, None).unwrap();
    // let mut servo2 = Servo::new(2, None, None, None).unwrap();

    // servo0.set_angle(0);
    // servo1.set_angle(90);
    // servo2.set_angle(0);
    // thread::sleep(Duration::from_millis(1000));

    let mut servo3 = Servo::new(3, None, None, None).unwrap();
    let mut servo4 = Servo::new(4, None, None, None).unwrap();
    let mut servo5 = Servo::new(5, None, None, None).unwrap();

    servo3.set_angle(90);
    servo4.set_angle(90);
    servo5.set_angle(0);
    thread::sleep(Duration::from_millis(1000));

    // let mut servo6 = Servo::new(6, None, None, None).unwrap();
    // let mut servo7 = Servo::new(7, None, None, None).unwrap();
    // let mut servo8 = Servo::new(8, None, None, None).unwrap();

    // servo6.set_angle(0);
    // servo7.set_angle(90);
    // servo8.set_angle(0);
    // thread::sleep(Duration::from_millis(1000));

    // let mut servo9 = Servo::new(9, None, None, None).unwrap();
    // let mut servo10 = Servo::new(10, None, None, None).unwrap();
    // let mut servo11 = Servo::new(11, None, None, None).unwrap();

    // servo9.set_angle(0);
    // servo10.set_angle(90);
    // servo11.set_angle(0);
    // thread::sleep(Duration::from_millis(1000));

    device::reset_mcu();
    println!("Reset MCU done!");

    // device::reset_mcu();
    // println!("Reset MCU done!");
}
