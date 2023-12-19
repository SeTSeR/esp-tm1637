#![no_std]
#![no_main]

use hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

use log::debug;

pub const BRIGHTNESS_MAX: u8 = 0x7;

const DISPLAY_MASK: u8 = 0x88;
const DISPLAY_OFF: u8 = 0x80;
const DISPLAY_ON: u8 = DISPLAY_MASK | BRIGHTNESS_MAX;
const AUTO_INC: u8 = 0x40;
const C0H: u8 = 0xC0;
const ENCODINGS: &[u8] = &[
    0b00111111, 0b00000110, 0b01011011, 0b01001111, 0b01100110, 0b01101101, 0b01111101, 0b00000111,
    0b01111111, 0b01101111,
];

pub struct TM1637<DIO, CLK, DL, E>
where
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
{
    // Time to wait before switching the lock in us.
    lock_time: u32,

    // Pin on which data input resides.
    dio_pin: DIO,

    // Pin on which clock input resides.
    clk_pin: CLK,

    // Delay timer
    delay: DL,
}

impl<DIO, CLK, DL, E> TM1637<DIO, CLK, DL, E>
where
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
{
    pub fn new(dio_pin: DIO, clk_pin: CLK, delay: DL) -> Result<TM1637<DIO, CLK, DL, E>, E> {
        let mut ret = TM1637 {
            lock_time: 3,
            dio_pin,
            clk_pin,
            delay,
        };
        ret.init()?;
        Ok(ret)
    }

    pub fn send_bytes(&mut self, data: &[u8], brightness: u8) -> Result<(), E> {
        self.send_iter(data.iter().map(|x| *x), brightness)
    }

    pub fn send_digits(&mut self, data: &[u8], clock_mode: bool, brightness: u8) -> Result<(), E> {
        self.send_iter(
            data.iter()
                .map(|x| ENCODINGS[*x as usize])
                .map(|x| x | (clock_mode as u8) << 7),
            brightness,
        )
    }

    pub fn send_number(&mut self, mut data: u32, brightness: u8) -> Result<(), E> {
        let mut base = 1;
        while base <= data {
            base *= 10;
        }
        base /= 10;
        self.send_iter(
            core::iter::from_fn(move || {
                if base > 0 {
                    let res = data / base;
                    data = data % base;
                    base /= 10;
                    Some(ENCODINGS[res as usize])
                } else {
                    None
                }
            }),
            brightness,
        )
    }

    fn send_iter<Iter: Iterator<Item = u8>>(
        &mut self,
        data: Iter,
        brightness: u8,
    ) -> Result<(), E> {
        self.start_send()?;
        self.send_byte(AUTO_INC)?;
        self.stop_send()?;

        self.start_send()?;
        self.send_byte(C0H)?;
        for b in data {
            self.send_byte(b)?;
        }
        self.stop_send()?;

        self.start_send()?;
        assert!(brightness <= BRIGHTNESS_MAX);
        let display_byte = DISPLAY_MASK | brightness;
        self.send_byte(display_byte)?;
        self.stop_send()
    }

    fn init(&mut self) -> Result<(), E> {
        self.dio_pin.set_high()?;
        self.clk_pin.set_high()?;

        self.delay.delay_us(self.lock_time);
        Ok(())
    }

    fn send_byte(&mut self, mut data: u8) -> Result<(), E> {
        debug!("Send byte {:#x}", data);
        for _ in 0..8 {
            let data_bit = data & 1;
            if data_bit == 1 {
                self.dio_pin.set_high()?;
            } else if data_bit == 0 {
                self.dio_pin.set_low()?;
            } else {
                unreachable!()
            }
            self.delay.delay_us(self.lock_time / 2);
            self.clk_pin.set_high()?;
            self.delay.delay_us(self.lock_time);
            self.clk_pin.set_low()?;
            self.delay.delay_us(self.lock_time / 2);
            data >>= 1;
        }

        self.dio_pin.set_high()?;
        self.wait_for_ack()?;

        self.delay.delay_us(self.lock_time / 2);

        Ok(())
    }

    fn wait_for_ack(&mut self) -> Result<(), E> {
        for i in 0..255 {
            if self.dio_pin.is_high()? {
                self.delay.delay_us(self.lock_time);
            } else {
                debug!("Found on {} tact", i);
                self.delay.delay_us(self.lock_time);
                self.clk_pin.set_high()?;
                self.delay.delay_us(self.lock_time);
                self.clk_pin.set_low()?;
                return Ok(());
            }
        }
        panic!("Timed out wait for ack!");
    }

    fn start_send(&mut self) -> Result<(), E> {
        self.clk_pin.set_high()?;
        self.dio_pin.set_high()?;
        self.delay.delay_us(self.lock_time);

        self.dio_pin.set_low()?;
        self.delay.delay_us(self.lock_time);
        self.clk_pin.set_low()?;
        self.delay.delay_us(self.lock_time / 2);

        Ok(())
    }

    fn stop_send(&mut self) -> Result<(), E> {
        self.clk_pin.set_low()?;
        self.delay.delay_us(self.lock_time);
        self.dio_pin.set_low()?;
        self.delay.delay_us(self.lock_time);

        self.clk_pin.set_high()?;
        self.delay.delay_us(self.lock_time);
        self.dio_pin.set_high()?;
        self.delay.delay_us(self.lock_time);

        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), E> {
        self.start_send()?;
        self.send_byte(DISPLAY_ON)?;
        self.stop_send()
    }

    fn disable(&mut self) -> Result<(), E> {
        self.start_send()?;
        self.send_byte(DISPLAY_OFF)?;
        self.stop_send()
    }
}

impl<DIO, CLK, DL, E> Drop for TM1637<DIO, CLK, DL, E>
where
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
{
    fn drop(&mut self) {
        let _ = self.disable();
    }
}
