//! A rust driver for agilent_4uhv
//!
//! TODO: Short description of what the driver does
//!
//! # Example
//!
//! TODO: High-level example
//! ```no_run
//! ```

//TODO: Uncomment the following line to enable warnings and missing docs
//#![deny(warnings, missing_docs)]

// TODO: Write an example with a Moxa TCPIP interface
// TODO: Allow for sending the channel number as part of the command package

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use instrumentrs::{InstrumentError, InstrumentInterface};

use measurements::Pressure;

use crate::{
    cmd_package::{CommandPackage, Data},
    read_package::ReadPackage,
};

mod cmd_package;
mod read_package;
mod utils;

/// High voltage state for the channels of the Agilent4Uhv.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum HvState {
    /// High voltage is off
    #[default]
    Off = 0,
    /// High voltage is on
    On = 1,
}

impl From<HvState> for Data {
    fn from(state: HvState) -> Self {
        let val: i64 = state as i64;
        Data::try_from(val).expect("Always within range")
    }
}

impl From<bool> for HvState {
    fn from(state: bool) -> Self {
        match state {
            false => HvState::Off,
            true => HvState::On,
        }
    }
}

impl Display for HvState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HvState::Off => "Off",
            HvState::On => "On",
        };
        write!(f, "{}", s)
    }
}

/// Units that are available on the Agilent4Uhv.
///
/// These are the units you can set the instrument to, however, all measured values are still
/// returned as [``Measurements``] units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Torr
    Torr = 0,
    /// Millibar - the default unit of the instrument
    #[default]
    #[allow(non_camel_case_types)]
    mBar = 1,
    /// Pascal
    Pa = 2,
}

impl From<Unit> for Data {
    fn from(unit: Unit) -> Self {
        let val: i64 = unit as i64;
        Data::try_from(val).expect("Always within range")
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Unit::Torr => "Torr",
            Unit::mBar => "mBar",
            Unit::Pa => "Pa",
        };
        write!(f, "{}", s)
    }
}
/// A rust driver for the Agilent4Uhv.
///
/// This driver provides functionality to control the Agilent/Agilent4Uhv.
///
/// # Example
/// TODO
/// ```no_run
/// ```
pub struct Agilent4Uhv<T: InstrumentInterface> {
    /// The [`InstrumentInterface`] to communicate with the instrument.
    interface: Arc<Mutex<T>>,
    /// The current unit of measurement.
    unit: Arc<Mutex<Unit>>,
    // The device address, usually 0. This is ignored for serial communication and only uses for
    // RS-485 communication.
    device_address: u8,
    /// The number of channels the instrument has, fixed at 4.
    num_channels: usize,
}

impl<T: InstrumentInterface> Agilent4Uhv<T> {
    /// Create a new Agilent4Uhv instance with the given instrument interface.
    ///
    /// This function uses device address 0 by default, as this is the default address for RS-485
    /// communication and completely ignored for serial communication.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn try_new(interface: T) -> Result<Self, InstrumentError> {
        Self::try_new_with_address(interface, 0)
    }

    /// Create a new Agilent4Uhv instance with the given instrument interface.
    ///
    /// For serial communication or to use the default device address of 0, please use the
    /// [`try_new`] method.
    ///
    /// Valid device addresses are 0 to 31. If an invalid address is provided, an error is
    /// returned.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    /// * `address` - The device address to use for RS-485 communication.
    pub fn try_new_with_address(interface: T, device_address: u8) -> Result<Self, InstrumentError> {
        if device_address > 31 {
            return Err(InstrumentError::IntValueOutOfRange {
                value: device_address.into(),
                min: 0,
                max: 31,
            });
        }

        let interface = Arc::new(Mutex::new(interface));

        let mut instrument = Agilent4Uhv {
            interface,
            unit: Arc::new(Mutex::new(Unit::default())),
            device_address,
            num_channels: 4,
        };
        instrument.update_unit()?;
        Ok(instrument)
    }

    /// Get a new channel with a given index for the Channel.
    ///
    /// Please note that channels are zero indexed.
    pub fn get_channel(&mut self, idx: usize) -> Result<Channel<T>, InstrumentError> {
        if idx >= self.num_channels {
            return Err(InstrumentError::ChannelIndexOutOfRange {
                idx,
                nof_channels: self.num_channels,
            });
        }
        Ok(Channel::new(
            idx,
            Arc::clone(&self.interface),
            self.device_address,
            Arc::clone(&self.unit),
        ))
    }

    /// Set the number of channels for the Agilent4Uhv.
    pub fn set_num_channels(&mut self, num: usize) -> Result<(), InstrumentError> {
        if !(1..5).contains(&num) {
            let num: i64 = num.try_into().unwrap_or(i64::MAX);
            return Err(InstrumentError::IntValueOutOfRange {
                value: num,
                min: 1,
                max: 4,
            });
        }
        self.num_channels = num;
        Ok(())
    }

    /// Get the model number of the instrument as a [`String`].
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        let resp = self.query(CommandPackage::new_read(319, self.device_address))?;
        resp.alphanumeric_pkg()
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: CommandPackage) -> Result<(), InstrumentError> {
        sendcmd(Arc::clone(&self.interface), cmd)
    }

    /// Query the instrument and return the response package.
    fn query(&mut self, cmd: CommandPackage) -> Result<ReadPackage, InstrumentError> {
        query(Arc::clone(&self.interface), cmd)
    }

    /// Get the current unit from the instrument.
    ///
    /// This updates the internally kept unit and returns a copy of it.
    pub fn get_unit(&mut self) -> Result<Unit, InstrumentError> {
        self.update_unit()?;
        let unit = self.unit.lock().expect("Mutex should not be poisoned");
        Ok(*unit)
    }

    /// Set the unit for the instrument.
    ///
    /// This sets a new unit for the instrument and, if successful, updates the internal unit
    /// representation to match the new unit.
    ///
    /// # Arguments
    /// - `unit`: The new unit to set for the instrument.
    pub fn set_unit(&mut self, unit: Unit) -> Result<(), InstrumentError> {
        let cmd = CommandPackage::new_write(600, unit.into(), self.device_address);
        self.sendcmd(cmd)?;
        {
            let mut guard = self.unit.lock().expect("Mutex should not be poisoned");
            *guard = unit;
        }
        Ok(())
    }

    /// Update the unit by querying the instrument for the current unit setting.
    pub fn update_unit(&mut self) -> Result<(), InstrumentError> {
        let resp = self.query(CommandPackage::new_read(600, self.device_address))?;
        let unit = match resp.int_pkg()? {
            0 => Unit::Torr,
            1 => Unit::mBar,
            2 => Unit::Pa,
            _ => {
                return Err(InstrumentError::ResponseParseError(
                    "Invalid unit received from instrument".into(),
                ));
            }
        };
        {
            let mut guard = self.unit.lock().expect("Mutex should not be poisoned");
            *guard = unit;
        }
        Ok(())
    }
}

impl<T: InstrumentInterface> Clone for Agilent4Uhv<T> {
    fn clone(&self) -> Self {
        Self {
            interface: self.interface.clone(),
            unit: self.unit.clone(),
            device_address: self.device_address,
            num_channels: self.num_channels,
        }
    }
}

/// Channel structure representing a single channel of the Agilent4Uhv.
///
/// **This structure can only be created through the [`Agilent4Uhv`] struct.**
///
/// Implementation of an individual channel and commands that go to it.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
    device_address: u8,
    unit: Arc<Mutex<Unit>>,
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the [`Agilent4Uhv`] struct.
    fn new(
        idx: usize,
        interface: Arc<Mutex<T>>,
        device_address: u8,
        unit: Arc<Mutex<Unit>>,
    ) -> Self {
        Channel {
            idx,
            interface,
            device_address,
            unit,
        }
    }

    /// Get the current high voltage state of the Channel.
    pub fn get_hv_state(&mut self) -> Result<HvState, InstrumentError> {
        let win = match self.idx {
            0 => 11,
            1 => 12,
            2 => 13,
            3 => 14,
            _ => unreachable!("Channel index should always be valid"),
        };
        let resp = self.query(CommandPackage::new_read(win, self.device_address))?;
        let state = match resp.int_pkg()? {
            0 => HvState::Off,
            1 => HvState::On,
            _ => {
                return Err(InstrumentError::ResponseParseError(
                    "Invalid HV state received from instrument".into(),
                ));
            }
        };
        Ok(state)
    }

    /// Set the high voltage state of the Channel.
    ///
    /// Arguments:
    /// - `state`: The new high voltage state to set for the channel.
    ///
    /// If a `NotAcknowledged("Data Type Error")` error is returned, the controller is likely not set
    /// connected to a pump and thus, the HV cannot be turned on.
    /// TEST: This needs to be tested with an actual instrument connected.
    pub fn set_hv_state(&mut self, state: HvState) -> Result<(), InstrumentError> {
        let win = match self.idx {
            0 => 11,
            1 => 12,
            2 => 13,
            3 => 14,
            _ => unreachable!("Channel index should always be valid"),
        };
        let cmd = CommandPackage::new_write(win, state.into(), self.device_address);
        self.sendcmd(cmd)
    }

    /// Read the pressure measurement from the channel.
    pub fn get_pressure(&mut self) -> Result<Pressure, InstrumentError> {
        let win = match self.idx {
            0 => 812,
            1 => 822,
            2 => 832,
            3 => 842,
            _ => unreachable!("Channel index should always be valid"),
        };
        let resp = self.query(CommandPackage::new_read(win, self.device_address))?;
        let val_str = resp.alphanumeric_pkg()?;
        let val = val_str.parse::<f64>().map_err(|_| {
            InstrumentError::ResponseParseError(format!("Cannot convert {} to f64.", val_str))
        })?;
        let pressure = {
            let unit = self.unit.lock().expect("Mutex should not be poisoned");
            match *unit {
                Unit::Torr => {
                    // HACK: Should be included in measurements
                    let val_pa = val * 133.32236842;
                    Pressure::from_pascals(val_pa)
                }
                Unit::mBar => Pressure::from_millibars(val),
                Unit::Pa => Pressure::from_pascals(val),
            }
        };
        Ok(pressure)
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: CommandPackage) -> Result<(), InstrumentError> {
        sendcmd(Arc::clone(&self.interface), cmd)
    }

    /// Query the instrument and return the response package.
    fn query(&mut self, cmd: CommandPackage) -> Result<ReadPackage, InstrumentError> {
        query(Arc::clone(&self.interface), cmd)
    }
}

impl<T: InstrumentInterface> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            interface: self.interface.clone(),
            device_address: self.device_address,
            unit: self.unit.clone(),
        }
    }
}

/// Send command function.
fn sendcmd<T: InstrumentInterface>(
    intf: Arc<Mutex<T>>,
    cmd: CommandPackage,
) -> Result<(), InstrumentError> {
    {
        let mut intf = intf.lock().expect("Mutex should not be poisoned");
        intf.write_raw(cmd.as_bytes())?;
    }
    read_package(intf)?.ack_pkg()
}

/// Query function.
fn query<T: InstrumentInterface>(
    intf: Arc<Mutex<T>>,
    cmd: CommandPackage,
) -> Result<ReadPackage, InstrumentError> {
    {
        let mut intf = intf.lock().expect("Mutex should not be poisoned");
        intf.write_raw(cmd.as_bytes())?;
    }
    read_package(intf)
}

/// Read one package from the instrument.
///
/// Reader reads individual bytes until it encounters an ETX byte, then it reads two more
/// (CRC). The ETX byte is `0x03`.
///
/// Returns: Result of a [`ReadPackage`] or an [`InstrumentError`].
fn read_package<T: InstrumentInterface>(
    intf: Arc<Mutex<T>>,
) -> Result<ReadPackage, InstrumentError> {
    let buf = {
        let mut intf = intf.lock().expect("Mutex should not be poisoned");
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            intf.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] == 0x03 {
                // ETX found, read two more bytes for CRC
                intf.read_exact(&mut byte)?;
                buf.push(byte[0]);
                intf.read_exact(&mut byte)?;
                buf.push(byte[0]);
                break;
            }
        }
        buf
    }; // make sure the lock is released here
    ReadPackage::try_new(&buf)
}
