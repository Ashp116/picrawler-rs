mod device;

fn main() {
    println!("Hello, world, I am picrawler!");
    device::reset_mcu();
    println!("Reset MCU done!")
}
