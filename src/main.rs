use std::{env, thread, time::Duration};


use crate::{robot::Robot};

mod device;
mod utils;
mod actuators;
mod robot;
mod robot_config;

fn main() {
    println!("Hello, world, I am picrawler!");
    device::reset_mcu();
    println!("Reset MCU done!");
    // thread::sleep(Duration::from_sdecs(1));

    let args: Vec<String> = env::args().collect();
    
    let config_path = args.iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("robot.yaml");

    
    // thread::sleep(Duration::from_secsd(1));

    // let I2c_bus = Arc::new(Mutex::new({
    //     let mut i2c = I2c::with_bus(1).unwrap();
    //     i2c.set_slave_address(0x14).unwrap();
    //     i2c
    // }));

    let mut robot = Robot::from_yaml(config_path.to_string()).unwrap();
    thread::sleep(Duration::from_millis(300));
    println!("{}", robot.name);
    let mut mul = 1.0;

    for _i in 0..100 {
        robot.set_servo_angle(90.0 * mul);
        thread::sleep(Duration::from_millis(200));
        mul = mul * -1.0;
    }
    

    // create all 12 first
    // let mut servo0 = Servo::new(Arc::clone(&I2c_bus), 0, None, None).unwrap();
    // let mut servo1 = Servo::new(Arc::clone(&I2c_bus), 1, None, None).unwrap();
    // let mut servo2 = Servo::new(Arc::clone(&I2c_bus), 2, None, None).unwrap();
    // let mut servo3 = Servo::new(Arc::clone(&I2c_bus), 3, None, None).unwrap();
    // let mut servo4 = Servo::new(Arc::clone(&I2c_bus), 4, None, None).unwrap();
    // let mut servo5 = Servo::new(Arc::clone(&I2c_bus), 5, None, None).unwrap();
    // let mut servo6 = Servo::new(Arc::clone(&I2c_bus), 6, None, None).unwrap();
    // let mut servo7 = Servo::new(Arc::clone(&I2c_bus), 7, None, None).unwrap();
    // let mut servo8 = Servo::new(Arc::clone(&I2c_bus), 8, None, None).unwrap();
    // let mut servo9 = Servo::new(Arc::clone(&I2c_bus), 9, None, None).unwrap();
    // let mut servo10 = Servo::new(Arc::clone(&I2c_bus), 10, None, None).unwrap();
    // let mut servo11 = Servo::new(Arc::clone(&I2c_bus), 11, None, None).unwrap();

    // thread::sleep(Duration::from_millis(100));

    // // then set all angles
    // servo0.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo1.set_angle(90);
    // thread::sleep(Duration::from_millis(20));
    // servo2.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo3.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo4.set_angle(90);
    // thread::sleep(Duration::from_millis(20));
    // servo5.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo6.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo7.set_angle(90);
    // thread::sleep(Duration::from_millis(20));
    // servo8.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo9.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
    // servo10.set_angle(90);
    // thread::sleep(Duration::from_millis(20));
    // servo11.set_angle(0);
    // thread::sleep(Duration::from_millis(20));
}
