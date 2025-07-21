use std::{net::Ipv4Addr, time::Duration};

use instrumentrs::SerialInstrument;

use pfeiffer_tpg36x::{EthernetConfig, Tpg36x};

fn main() {
    // Create a serial port builder using the `serialport::new` function.
    let spb = serialport::new("/dev/ttyUSB0", 9600).timeout(Duration::from_secs(3));

    // Define the serial interface using the serial port builder object.
    let interface = SerialInstrument::try_new(spb).expect("Instrument must be available.");

    // Now we can open the DigOutBox with the serial interface.
    let mut inst = Tpg36x::try_new(interface).unwrap();

    // Query and print the name of the instrument
    println!("Instrument name: {}", inst.get_name().unwrap());

    // Set Ethernet config to dynamic
    inst.set_ethernet_config(EthernetConfig::new_dynamic())
        .unwrap();
    // Get Ethernet config
    println!("Ethernet config: {:?}", inst.get_ethernet_config().unwrap());

    // Set Ethernet config to static
    let eth_conf = EthernetConfig::new_static(
        Ipv4Addr::new(192, 168, 127, 42),
        Ipv4Addr::new(255, 255, 255, 0),
        Ipv4Addr::new(192, 168, 127, 10),
    );
    inst.set_ethernet_config(eth_conf).unwrap();
    // Get Ethernet config
    println!("Ethernet config: {:?}", inst.get_ethernet_config().unwrap());
}
