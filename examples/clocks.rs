#![no_std]
#![no_main]

use esp32_tm1637::TM1637;

use esp_backtrace as _;
use esp_hal::entry;
use esp_hal::prelude::*;
use esp_println::logger::init_logger;

#[derive(Debug)]
pub enum ClocksError {
    Overflow
}

pub struct Clocks {
    hours: u8,

    minutes: u8,
}

impl Clocks {
    pub fn new(hours: u8, minutes: u8) -> Result<Clocks, ClocksError> {
	if hours < 24 && minutes < 60 {
	    Ok(Clocks {
		hours, minutes
	    })
	} else {
	    Err(ClocksError::Overflow)
	}
    }

    pub fn tick(&mut self) {
	self.minutes += 1;
	if self.minutes == 60 {
	    self.minutes = 0;
	    self.hours += 1;
	}
	if self.hours == 24 {
	    self.hours = 0;
	}
    }

    pub fn to_value(&self) -> [u8; 4] {
	[
	    self.hours / 10,
	    self.hours % 10,
	    self.minutes / 10,
	    self.minutes % 10,
	]
    }
}

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    let peripherals = esp_hal::peripherals::Peripherals::take();
    use esp_hal::clock::{ClockControl, CpuClock};

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let io = esp_hal::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut delay = esp_hal::delay::Delay::new(&clocks);

    let mut display = TM1637::new(
        io.pins.gpio22.into_open_drain_output(),
        io.pins.gpio23.into_open_drain_output(),
        delay,
    )
    .unwrap();

    let mut clocks = Clocks::new(20, 00).unwrap();

    loop {
	display.send_digits(&clocks.to_value()).unwrap();
	delay.delay_ms(60 * 1000u32);
	clocks.tick();
    }
}
