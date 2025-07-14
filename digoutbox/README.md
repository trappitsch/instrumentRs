# Rust controller class for the DigOutBox

This is the rust controller package for the 
[DigOutBox](https://digoutbox.readthedocs.io/latest/) project.

To run the examples, you can use the following command:

```bash
cargo run --example serial-blocking --feautres examples
``` 

Don't forget to add the `-F examples` flag to enable the examples feature, 
as the examples require more dependencies than the main package.
Above example will run the communications using the 
[`serialport`](https://crates.io/crates/serialport) crate in blocking mode.

