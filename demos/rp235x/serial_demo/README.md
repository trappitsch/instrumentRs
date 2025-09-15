# `rp235x` serial demo

This demo uses a Raspberry Pi Pico 2 microcontroller to simulate a serial instrument. 

The commands are modeled after a SCPI instrument, and the following are available:

- `*IDN?`: Returns a instrument identification string.
- `LED x`: Where `x` is either `0` or `1`, turns the onboard LED off or on.
- `LED?`: Queries the state of the onboard LED, returning either `LED 0` (off) or `LED 1` (on).

The terminator for all commands is `\n`.

## Getting started

This is currently setup to be flashed using a debug probe and `probe-rs`. 
