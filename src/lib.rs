#![no_std]
#![no_main]
use core::convert::Infallible;
        
use esp_backtrace as _;
use esp_println::println;

use hal::{blocking::delay::DelayUs, digital::v2::OutputPin};

const DISPLAY_OFF: u8 = 0x80;
const DISPLAY_ON: u8 = 0x8F;
const AUTO_INC: u8 = 0x40;
const C0H: u8 = 0xC0;
const ENCODINGS: u8 = [ 0b11111100,
			1, 2, 3, 4, 5, 6, 7, 8, 9 ];

pub struct TM1637<DIO, CLK, DL, E>
where
    DIO: OutputPin<Error = E> + esp_hal::gpio::InputPin + esp_hal::gpio::OutputPin,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
    E: From<Infallible>,
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
    DIO: OutputPin<Error = E> + esp_hal::gpio::InputPin + esp_hal::gpio::OutputPin,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
    E: From<Infallible>,
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

    pub fn send_bytes(&mut self, data: &[u8]) -> Result<(), E> {
        self.start_send()?;
        self.send_byte(AUTO_INC)?;
        self.stop_send()?;
        self.start_send()?;
        self.send_byte(C0H)?;
        for b in data {
            println!("Send byte {}", *b);
            self.send_byte(*b)?;
        }
        self.stop_send()?;
        self.start_send()?;
        self.send_byte(DISPLAY_ON)?;
        self.stop_send()
    }
    
    fn init(&mut self) -> Result<(), E> {
        self.dio_pin.set_high()?;
        self.clk_pin.set_high()?;

        self.dio_pin.set_to_input();
        assert!(self.dio_pin.is_input_high());
        self.dio_pin.set_to_push_pull_output();
        self.delay.delay_us(self.lock_time);
        Ok(())
    }

    fn send_byte(&mut self, mut data: u8) -> Result<(), E> {
        for _ in 0..7 {
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
        self.delay.delay_us(self.lock_time / 2);
        self.clk_pin.set_high()?;
        self.delay.delay_us(self.lock_time);
        self.clk_pin.set_low()?;
        self.delay.delay_us(self.lock_time / 2);
        self.wait_for_ack()?;
            
        self.delay.delay_us(self.lock_time / 2);
        
        Ok(())
    }

    fn wait_for_ack(&mut self) -> Result<(), E> {
        self.dio_pin.set_to_input();
        for i in 0..255 {
            if self.dio_pin.is_input_high() {
                self.delay.delay_us(self.lock_time);
            } else {
                println!("Found on {} tact", i);
                self.delay.delay_us(self.lock_time);
                self.clk_pin.set_high()?;
                self.delay.delay_us(self.lock_time);
                self.clk_pin.set_low()?;
                return Ok(())
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
        self.dio_pin.set_to_push_pull_output();
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
    DIO: OutputPin<Error = E> + esp_hal::gpio::InputPin + esp_hal::gpio::OutputPin,
    CLK: OutputPin<Error = E>,
    DL: DelayUs<u32>,
    E: From<Infallible>,
{
    fn drop(&mut self) {
	let _ = self.disable();
    }
}
