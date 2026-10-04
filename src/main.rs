use std::{env, thread, time::{Duration, Instant}};

use crate::{camera::cam_mgr::cam_mgr, robot::Robot};

mod device;
mod _utils;
mod actuators;
mod actuator_group;
mod robot;
mod robot_config;
mod telemetry;
mod webui;
mod camera;

fn main() {
    println!("Hello world, I am picrawler!");
    device::reset_mcu();
    println!("Reset MCU done!");

    let mut cam_mgr = cam_mgr::new();

    for i in cam_mgr.list_cameras() {
        println!("cam {} \n ", i.display_name());
        println!("{:?}", i.properties());
    }

    if let Some(cam) = cam_mgr.default_camera() {
        println!("camera: using {}", cam.display_name());
        cam_mgr.set_main_cam(&cam.display_name());
        match cam_mgr.start(640, 480, 30) {
            Ok(()) => match cam_mgr.grab_frame(10000) {
                Ok(f) => println!("camera: got {}x{} frame ({} bytes)", f.width, f.height, f.data.len()),
                Err(e) => eprintln!("camera: {}", e),
            },
            Err(e) => eprintln!("camera: failed to start: {}", e),
        }
    }

    let args: Vec<String> = env::args().collect();

    let config_path = args.iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("robot.yaml");

    let mut robot = Robot::from_yaml(config_path.to_string()).unwrap();
    thread::sleep(Duration::from_millis(300));
    println!("{}", robot.name);

    let telemetry = if robot.config.telemetry.enable {
        match telemetry::TelemetryServer::start(&robot.config.telemetry) {
            Ok(server) => Some(server),
            Err(e) => {
                eprintln!("telemetry: failed to start: {}", e);
                None
            }
        }
    } else {
        None
    };

    if robot.config.telemetry.enable && robot.config.telemetry.webui.enable {
        let config_json = webui::config_json(&robot.config);
        if let Err(e) = webui::start(&robot.config.telemetry.webui, config_json) {
            eprintln!("webui: failed to start: {}", e);
        }
    }

    // set the target once; the tick loop below steps every servo toward it
    let mut mul = 1.0;
    robot.set_servo_angle(45.0 * mul);

    let mut last_tick = Instant::now();
    let mut last_vbat = Instant::now();
    let mut battery_v: Option<f32> = None;
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
                    //println!("all servos reached target ({})", 45.0 * mul);
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
                Ok(v) => {
                    //println!("battery: {:.2}V", v);
                    battery_v = Some(v);
                }
                Err(e) => eprintln!("battery read failed: {:?}", e),
            }
        }

        if let Some(server) = &telemetry {
            server.publish(&robot, battery_v);
        }

        thread::sleep(Duration::from_millis(10));
    }
}
