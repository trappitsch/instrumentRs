# InstrumentRs

The InstrumentRs library provides standardized interfaces to talk to scientific equipment via
various different ports. To do so, it provides an `InstrumentInterface` trait and its
implementations. Furthermore, we also provide an `InstrumentError` error type that instrument
drivers should return.

Furthermore, this repository also contains drivers for instruments that are (1) written using 
InstrumentRs and (2) are maintained by us. Have a look at the various folders for more information.

## Inspiration

InstrumentRs is heavily inspired by [instrumentkit](https://github.com/instrumentkit/InstrumentKit).

## Status

This project is currently under active development and (breaking) changes might occure fast. If
you are interested in using this project and/or contributing, please get in touch by raising an
issue on GitHub. This would also be super valuable as we would learn how it is used, what the
need is, etc.

## Instrument driver template

If you would like to write a driver for an instrument, we provide a 
[`cargo-generate`](https://github.com/cargo-generate/cargo-generate) template 
that you can use to get started quickly. To use it, install 
[`cargo-generate`](https://github.com/cargo-generate/cargo-generate) and run the
following command:

```bash
cargo generate --git https://github.com/trappitsch/instrumentRs
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
