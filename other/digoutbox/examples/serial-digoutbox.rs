use instrumentrs::SerialInterface;

use digoutbox::DigOutBox;

fn main() {
    // Create a serial port builder using the `serialport::new` function.
    let port = "/dev/ttyACM0";
    let baud = 9600;

    // Let us get an instrument with the serial interface.
    let serial_inst = SerialInterface::simple(port, baud).expect("Failed to open serial port");

    // Now we can open the DigOutBox with the serial instrument.
    let mut inst = DigOutBox::new(serial_inst);

    // Query and print the name of the instrument
    println!("Instrument name: {}", inst.get_name().unwrap());

    // set outputs of channels 0 to 9 to true
    for i in 0..10 {
        let mut channel = inst.get_channel(i).unwrap();
        channel.set_output(true).unwrap();
    }

    // Query the status of channel 7, then toggle it, then query it again
    let mut ch7 = inst.get_channel(7).unwrap();
    let ch7_status = ch7.get_output().unwrap();
    println!("Channel 7 output status: {ch7_status}");
    ch7.set_output(!ch7_status).unwrap();
    println!(
        "Channel 7 output status after toggle: {}",
        ch7.get_output().unwrap()
    );

    // Print the status of all outputs as a vector of boolean values
    println!(
        "All output are set do: {:?}",
        inst.get_all_outputs().unwrap()
    );

    // Print out the interlock and software control status
    println!("Interlock status: {}", inst.get_interlock_status().unwrap());
    println!(
        "Software control status: {}",
        inst.get_software_control_status().unwrap()
    );
}
