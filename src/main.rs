use std::{env, thread, time::{Duration, Instant}};


use crate::{actuator_group::ServoGroup, actuators::{Servo, servo}, device::reset_mcu, robot::Robot};

mod device;
mod _utils;
mod actuators;
mod actuator_group;
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

    reset_mcu();
    // give the MCU time to boot before ServoGroup::new hits it with timer-config writes
    thread::sleep(Duration::from_millis(200));

    let mut robot = Robot::from_yaml(config_path.to_string()).unwrap();
    thread::sleep(Duration::from_millis(300));
    println!("{}", robot.name);
    let mut mul = 1.0;

    // call ONCE to set target
    robot.set_servo_angle(45.0);

    // call every frame to advance toward it, using the real elapsed time since the last tick
    let mut last_tick = Instant::now();
    loop {
        let now = Instant::now();
        let dt_ms = now.duration_since(last_tick).as_secs_f32() * 1000.0;
        last_tick = now;

        robot.tick(dt_ms);
        println!("dt={:.2}ms ch0_angle={:.2}", dt_ms, robot.get_servo_angle(0).unwrap_or(f32::NAN));
        thread::sleep(Duration::from_millis(10));
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
