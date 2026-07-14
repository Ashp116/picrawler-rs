use std::{env, thread, time::{Duration, Instant}};

use crate::robot::Robot;

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

    let args: Vec<String> = env::args().collect();

    let config_path = args.iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("robot.yaml");

    let mut robot = Robot::from_yaml(config_path.to_string()).unwrap();
    thread::sleep(Duration::from_millis(300));
    println!("{}", robot.name);

    // set the target once; the tick loop below steps every servo toward it
    let mut mul = 1.0;
    robot.set_servo_angle(45.0 * mul);

    let mut last_tick = Instant::now();
    let mut last_vbat = Instant::now();
    // when the servos reach the target, dwell for 1s (without blocking the tick
    // loop) and then swing to the opposite side
    let mut dwell_until: Option<Instant> = None;
    loop {
        let now = Instant::now();
        let dt_ms = now.duration_since(last_tick).as_secs_f32() * 1000.0;
        last_tick = now;

        robot.tick(dt_ms);

        if robot.all_servos_at_target() {
            match dwell_until {
                None => {
                    println!("all servos reached target ({})", 45.0 * mul);
                    dwell_until = Some(Instant::now() + Duration::from_millis(100));
                }
                Some(t) if Instant::now() >= t => {
                    mul *= -1.0;
                    robot.set_servo_angle(45.0 * mul);
                    dwell_until = None;
                }
                Some(_) => {}
            }
        }

        if last_vbat.elapsed() >= Duration::from_secs(1) {
            last_vbat = Instant::now();
            match device::get_battery_voltage() {
                Ok(v) => println!("battery: {:.2}V", v),
                Err(e) => eprintln!("battery read failed: {:?}", e),
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}
