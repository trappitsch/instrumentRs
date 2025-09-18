//! This is a demo of a simple USB serial device that can be used as a demo instrument for
//! InstrumentRs.
//!
//! The example is based on the `rp-hal` USB serial example, which can be found here:
//! https://github.com/rp-rs/rp-hal/blob/main/rp235x-hal-examples/src/bin/usb.rs

#![no_std]
#![no_main]

use embedded_hal::digital::{OutputPin, StatefulOutputPin};
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use defmt::{info, warn};
use defmt_rtt as _;
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal::{self as hal, Sio, gpio};

const TERMINATOR: u8 = b'\n';

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // set up gpio for led pins
    let sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let led_pin = pins.gpio25.into_push_pull_output();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("InstrumentRs")
            .product("Serial Instrument Demo")
            .serial_number("123456789")])
        .unwrap()
        .max_packet_size_0(64)
        .unwrap()
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    let mut data_buf = DataBuffer::new();
    let mut command_processor = CommandProcessor::new(led_pin);
    loop {
        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut read_buf = [0u8; 64];
            match serial.read(&mut read_buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    data_buf.add(&read_buf[..count]);
                    if let Some(command) = data_buf.get_command() {
                        let answer = command_processor.process(command);
                        match answer {
                            CommandAnswer::Ok => info!("Command OK"),
                            CommandAnswer::Error => warn!("Command ERROR"),
                            _ => {
                                serial.write(answer.into()).unwrap();
                            }
                        };
                        // let _ = serial.write(command);
                        data_buf.clear();
                    }
                }
            }
        }
    }
}

/// Process commands
struct CommandProcessor {
    led: gpio::Pin<gpio::bank0::Gpio25, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>,
}

impl CommandProcessor {
    fn new(
        led: gpio::Pin<gpio::bank0::Gpio25, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>,
    ) -> Self {
        Self { led }
    }

    fn process(&mut self, command: &[u8]) -> CommandAnswer {
        let command = command.trim_ascii();
        if command.starts_with(b"LED") {
            return self.led_handler(command[3..].trim_ascii());
        } else if command.starts_with(b"*IDN?") {
            return CommandAnswer::Id;
        }
        CommandAnswer::Error
    }

    fn led_handler(&mut self, state: &[u8]) -> CommandAnswer {
        match state {
            b"1" => self.led.set_high().unwrap(),
            b"0" => self.led.set_low().unwrap(),
            b"?" => {
                return CommandAnswer::LedState(self.led.is_set_high().unwrap());
            }
            _ => return CommandAnswer::Error,
        };
        CommandAnswer::Ok
    }
}

enum CommandAnswer {
    Ok,
    Error,
    LedState(bool),
    Id,
}

impl From<CommandAnswer> for &[u8] {
    fn from(answer: CommandAnswer) -> Self {
        match answer {
            CommandAnswer::Ok => b"OK\n",
            CommandAnswer::Error => b"ERR\n",
            CommandAnswer::LedState(state) => {
                if state {
                    b"LED 1\n"
                } else {
                    b"LED 0\n"
                }
            }
            CommandAnswer::Id => b"InstrumentRs Serial Instrument Demo\n",
        }
    }
}

/// Structure for data buffer
struct DataBuffer {
    buf: [u8; 64],
    pos: usize,
}

impl DataBuffer {
    /// Create a new, empty buffer
    fn new() -> Self {
        Self {
            buf: [0u8; 64],
            pos: 0,
        }
    }

    /// Add data to the buffer, return number of bytes added.
    ///
    /// If the buffer is full, just drop the rest of the data and return how many bytes were added.
    fn add(&mut self, data: &[u8]) -> usize {
        let space = self.buf.len() - self.pos;
        let last_index = if data.len() < space {
            data.len()
        } else {
            space
        };
        self.buf[self.pos..self.pos + last_index].copy_from_slice(&data[..last_index]);
        self.pos += last_index;
        last_index
    }

    /// Check if we have a full command and if so, return it.
    fn get_command(&self) -> Option<&[u8]> {
        if self.pos > 0 && self.buf[self.pos - 1] == TERMINATOR {
            Some(&self.buf[..self.pos])
        } else {
            None
        }
    }

    /// Clear the buffer
    fn clear(&mut self) {
        self.pos = 0;
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"USB Serial Demo instrument"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

// End of file
