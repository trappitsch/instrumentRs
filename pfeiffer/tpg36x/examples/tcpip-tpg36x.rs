use instrumentrs::TcpIpInterface;

use pfeiffer_tpg36x::{PressureUnit, SensorStatus, Tpg36x, Tpg36xMeasurement};

fn main() {
    // Define the TCP/IP instrument interface using `TcpIpInterface`.
    let tcpip_inst = TcpIpInterface::simple("192.168.127.42:8000").unwrap();

    // Now we can open the TPG36x with the TCP/IP instrument.
    let mut inst = Tpg36x::try_new(tcpip_inst).unwrap();

    // Query and print the name of the instrument
    println!("Instrument name: {}", inst.get_name().unwrap());

    // Get Ethernet config
    println!(
        "Ethernet config:\n{}\n\n",
        inst.get_ethernet_config().unwrap()
    );

    // get the MAC address:
    println!("MAC address: {}", inst.get_mac_address().unwrap());

    // Set the unit of measurement to millibars
    inst.set_unit(PressureUnit::mBar).unwrap();
    println!("Unit: {}", inst.get_unit().unwrap());

    // Get the first channel and read the pressure
    let mut ch0 = inst.get_channel(0).unwrap();
    let pressure = ch0.get_pressure();
    let val = match pressure {
        Ok(Tpg36xMeasurement::Pressure(p)) => p,
        _ => panic!("I'm only dealing with pressure measurements here! {pressure:?}"),
    };

    println!("Pressure channel 1: {}", val.as_millibars());

    // Turn the second channel on and get its status first, then the pressure
    let mut ch1 = inst.get_channel(1).unwrap();
    ch1.set_status(SensorStatus::On).unwrap();

    println!("Channel 2 pressure: {}", ch1.get_pressure().unwrap());
}
