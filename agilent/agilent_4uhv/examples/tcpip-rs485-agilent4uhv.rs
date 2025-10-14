//! This example demonstrates how to connect to an Agilent 4UHV controller when it is connected to
//! a network to serial converter, i.e., a Moxa NPort.
//!
//! The Moxa in this case has the communication setup to talk via RS-485 with the device at a
//! non-default address. Our interface on the other hand is simply TCP/IP with an IP address and port.
//!
//! NOTE: This is currently untested as I don't have access to such a setup.

use instrumentrs::TcpIpInterface;

use agilent_4uhv::{Agilent4Uhv, Unit};

fn main() {
    // Define the TCP/IP instrument interface using the `simple` method.
    let tcpip_inst = TcpIpInterface::simple("192.168.1.2:4001").unwrap();

    // Now we can open the Agilent4Uhv with the serial interface.
    let mut inst = Agilent4Uhv::try_new_with_address(tcpip_inst, 0x02).unwrap();

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
