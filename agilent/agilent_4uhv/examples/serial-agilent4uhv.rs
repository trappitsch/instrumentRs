//! This example demonstrates how to connect to an Agilent 4UHV controller when it is connected to
//! a serial interface, e.g., via a USB to serial adapter. We need to specify the serial port and
//! the baud rate to use. The instrument defaults to 9600 baud.

use instrumentrs::SerialInterface;

use agilent_4uhv::{Agilent4Uhv, Unit};

fn main() {
    let port = "/dev/ttyUSB0";
    let baud = 9600;

    // Define the serial instrument interface using the `simple` method.
    let serial_inst = SerialInterface::simple(port, baud).expect("Failed to open serial port");

    // Now we can open the Agilent4Uhv with the serial interface.
    let mut inst = Agilent4Uhv::try_new(serial_inst).unwrap();

    println!("Unit currently set: {}", inst.get_unit().unwrap());

    // Query and print the name of the instrument
    println!("Instrument model number: {}", inst.get_name().unwrap());

    // Set the unit to mbar
    // NOTE: If below `unwrap()` panics with a `NotAcknowledged("Win disabled")` error, the
    // controller is likely not set to `SERIAL` mode.

    inst.set_unit(Unit::mBar).unwrap();
    println!("Unit currently set: {}", inst.get_unit().unwrap());

    // Get some channels
    let mut ch1 = inst.get_channel(0).unwrap();
    let mut ch3 = inst.get_channel(2).unwrap();

    // Read the high voltage state from each channel
    println!("Channel 1 HV state: {}", ch1.get_hv_state().unwrap());
    println!("Channel 3 HV state: {}", ch3.get_hv_state().unwrap());

    // Read the pressure from each channel
    println!("Channel 1 pressure: {}", ch1.get_pressure().unwrap());
    println!("Channel 3 pressure: {}", ch3.get_pressure().unwrap());
}
