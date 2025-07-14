use std::time::Duration;

use instrumentrs::SerialInstrument;

use digoutbox::DigOutBox;

fn main() {
    // Create a serial port builder using the `serialport::new` function.
    let spb = serialport::new("/dev/ttyACM0", 9600).timeout(Duration::from_secs(3));

    // Define the serial interface using the serial port builder object.
    let interface = SerialInstrument::try_new(spb).expect("Instrument must be available.");

    // Now we can open the DigOutBox with the serial interface.
    let mut inst = DigOutBox::new(interface);

    // query and print the name of the instrument
    println!("Instrument name: {}", inst.get_name().unwrap());

    // set outputs of channels 0 to 9 to to true
    for i in 0..10 {
        let mut channel = inst.get_channel(i).unwrap();
        channel.set_output(true).unwrap();
    }

    // Query the status of channel 7, then toggle it, then query it again
    let mut ch7 = inst.get_channel(7).unwrap();
    let ch7_status = ch7.get_output().unwrap();
    println!("Channel 7 output status: {ch7_status}");
    ch7.set_output(!ch7_status).unwrap();
    println!("Channel 7 output status after toggle: {}", ch7.get_output().unwrap());

    // Print the status of all outputs as a vector of booleans
    println!("All output are set do: {:?}", inst.get_all_outputs().unwrap());

    // Print out the interlock and software control status
    println!("Interlock status: {}", inst.get_interlock_status().unwrap());
    println!("Software control status: {}", inst.get_software_control_status().unwrap());
}

