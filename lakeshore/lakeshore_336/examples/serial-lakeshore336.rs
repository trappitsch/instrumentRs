use lakeshore_336::{Lakeshore336, SerialInterfaceLakeshore};

fn main() {
    let port = "/dev/ttyUSB0";

    // Get our serial instrument interface
    let serial_inst = SerialInterfaceLakeshore::simple(port).expect("Failed to open serial port");

    // Now we can open the Lakeshore336 with the serial interface.
    let mut inst = Lakeshore336::try_new(serial_inst).unwrap();
    println!("Instrument ID: {}", inst.get_name().unwrap());

    // Get channel A and channel C
    let mut cha = inst.get_channel(0).unwrap();
    let mut chc = inst.get_channel(1).unwrap();

    // Query and print the temperature of channel A and channel C
    println!("Channel A temperature: {:?}", cha.get_temperature());
    println!("Channel C temperature: {:?}", chc.get_temperature());
}
