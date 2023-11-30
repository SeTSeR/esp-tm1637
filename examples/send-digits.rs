#![no_std]
#![no_main]

use esp32_tm1637::TM1637;

use esp_backtrace as _;
use esp_hal::entry;
use esp_hal::prelude::*;
use esp_println::logger::init_logger;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    let peripherals = esp_hal::peripherals::Peripherals::take();
    use esp_hal::clock::{ClockControl, CpuClock};

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let io = esp_hal::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let delay = esp_hal::delay::Delay::new(&clocks);

    let mut display = TM1637::new(
        io.pins.gpio22.into_open_drain_output(),
        io.pins.gpio23.into_open_drain_output(),
        delay,
    )
    .unwrap();
    display.send_number(1432).unwrap();

    loop {}
}
