#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::{logger::init_logger, println};
use hal::{
  clock::{ClockControl, CpuClock},
  delay::Delay,
  gpio::{AnyPin, Output, PushPull},
  peripherals::*,
  prelude::*,
  timer::TimerGroup,
  Rtc, IO,
};

const DISPLAY_OFF: u8 = 0x80;
const DISPLAY_ON: u8 = 0x88;
const CLOCK_LOCK_TIME: u8 = 3;
const DIO_PIN: u8 = 23;
const CLK_PIN: u8 = 22;

pub enum SettingType {
  DataSetting = 1,
  DisplayControlSetting,
  AddressSetting,
}

pub struct TM1637 {
  // Time to wait before switching the lock in us.
  lock_time: u8,

  // Pin on which data input resides.
  dio_pin: AnyPin<Output<PushPull>>,

  // Pin on which clock input resides.
  clk_pin: AnyPin<Output<PushPull>>
}

impl TM1637 {
  pub fn new(io: IO) -> TM1637 {
    let mut ret = TM1637 {
      lock_time: 3,
      dio_pin: io.pins.gpio23.into_push_pull_output().degrade(),
      clk_pin: io.pins.gpio22.into_push_pull_output().degrade(),
    };
    ret.dio_pin.set_high().unwrap();
    ret.clk_pin.set_high().unwrap();

    ret
  }
}

#[entry]
fn main() -> ! {
  init_logger(log::LevelFilter::Info);

  let peripherals = Peripherals::take();

  let system = peripherals.DPORT.split();
  let mut peripheral_clock_control = system.peripheral_clock_control;
  let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
  let mut rtc = Rtc::new(peripherals.RTC_CNTL);

  let timer = TimerGroup::new(peripherals.TIMG1, &clocks, &mut peripheral_clock_control).timer0;

  let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

  loop {
  }
}
