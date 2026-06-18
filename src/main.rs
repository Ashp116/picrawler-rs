mod device;
mod utils;

fn main() {
    println!("Hello, world, I am picrawler!");
    //device::reset_mcu();
    println!("Reset MCU done!");

    println!("{}v Battery voltage", device::get_battery_voltage().unwrap());
}
