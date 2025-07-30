use core::clone::Clone;

use std::thread;
use std::time::Duration;

use digoutbox::DigOutBox;
use instrumentrs::SerialInterface;

const PORT: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 9600;

fn main() {
    let interface = SerialInterface::simple(PORT, BAUD_RATE).unwrap();
    let mut inst = DigOutBox::new(interface);

    // Create a new channel where the interface is a clone of the original interface, which of
    // course is wrapped in an `Arc`.
    let mut ch0 = inst.get_channel(0).unwrap();

    // You can send the channel by cloning it. This is low cost, as it will just increase the
    // reference count on the `Arc`.
    let mut ch = ch0.clone();
    let t1 = thread::spawn(move || {
        for _ in 0..10 {
            println!("Thread 1: Channel 0 is on: {:?}", ch.get_output());
            thread::sleep(Duration::from_secs(2));
        }
    });

    // Send a second channel 0 to another thread and toggle it every 2 seconds 10 times.
    let t2 = thread::spawn(move || {
        for _ in 0..10 {
            let current = ch0.get_output().unwrap();
            ch0.set_output(!current).unwrap();
            println!("Thread 2: Toggled channel 0.");
            thread::sleep(Duration::from_secs(2));
        }
    });

    // Send the whole instrument to a thread and turn 11 seconds later all channels on.
    let t3 = {
        let mut inst = inst.clone();
        thread::spawn(move || {
            for i in 0..16 {
                let mut ch = inst.get_channel(i).unwrap();
                ch.set_output(true).unwrap();
            }
        })
    };

    thread::sleep(Duration::from_millis(500));
    // Read status of all channels every 2 seconds 10 times, but make sure the other two threads
    // have started fine.
    for _ in 0..10 {
        println!(
            "All channels read from main thread: {:?}",
            inst.get_all_outputs()
        );
        thread::sleep(Duration::from_secs(2));
    }

    // wait for threads to finish, then call it a day.
    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();

    inst.all_off().unwrap();
    println!("All off now and done.");
}
