use std::net::Ipv4Addr;

use instrumentrs::SerialInterface;

use pfeiffer_tpg36x::{EthernetConfig, Tpg36x};

fn main() {
    let port = "/dev/ttyUSB0";
    let baud = 9600;

    // Define the serial instrument interface using the `simple` method.
    let serial_inst = SerialInterface::simple(port, baud).expect("Failed to open serial port");

    // Now we can open the TPG36x with the serial interface.
    let mut inst = Tpg36x::try_new(serial_inst).unwrap();

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
