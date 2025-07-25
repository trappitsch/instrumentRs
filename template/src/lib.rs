//! A rust driver for {{ project-name }}
//!
//! TODO: Short description of what the driver does
//!
//! # Example
//!
//! TODO: High-level example
//! ```no_run
//! ```
{% assign noc = num_channels | abs %}
//TODO: Uncomment the following line to enable warnings and missing docs
//#![deny(warnings, missing_docs)]

use std::{fmt::Display, sync::{Arc, Mutex}};

use instrumentrs::{InstrumentError, InstrumentInterface};

{% if units -%}
/// Units that are available on the {{ device }}.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// TODO: Document all units and put them here
    #[default]
    Kg,
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!();
    }
}
{% endif -%}

/// A rust driver for the {{ device }}.
///
/// This driver provides functionality to control the {{ manufacturer }}/{{ device }}.
///
/// # Example
/// TODO
/// ```no_run
/// ``` 
pub struct {{ device | upper_camel_case }}<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
    {% if units -%}
    unit: Arc<Mutex<Unit>>, // TODO: Replace with actual unit type
    {% endif -%}
    {% if noc > 1 -%}
    num_channels: usize,
{% endif -%}
}

impl<T: InstrumentInterface> {{ device | upper_camel_case }}<T> {
    /// Create a new {{ device }} instance with the given instrument interface.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn try_new(interface: T) -> Result<Self, InstrumentError> {
        {% if terminator != "\n" -%}
        let mut intf = interface;
        intf.set_terminator("{{ terminator }}");
        let interface = Arc::new(Mutex::new(intf));
        {% else -%}
        let interface = Arc::new(Mutex::new(interface));
        {% endif -%}

        {% if units %}
        let mut instrument = {{ device | upper_camel_case }} {
        {% else -%}
        let instrument = {{ device | upper_camel_case }} {
            {% endif -%}
            interface,
            {% if units -%}
            unit: Arc::new(Mutex::new(Unit::Kg)), // TODO: Replace with actual unit type
            {% endif -%}
            {% if noc > 1 -%}
            num_channels: {{ num_channels }},
            {% endif -%}
        };
        {% if units -%}
        instrument.update_unit()?;
        {% endif -%}
        Ok(instrument)
    }

    {% if noc > 1 -%}
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
            {% if units -%}
            Arc::clone(&self.unit),
            {% endif -%}
        ))
    }

    /// Set the number of channels for the {{ device }}.
    pub fn set_num_channels(&mut self, num: usize) -> Result<(), InstrumentError> {
        if !(1..3).contains(&num) {  // TODO: Adjust range as needed!
            let num: i64 = num.try_into().unwrap_or(i64::MAX);
            return Err(InstrumentError::IntValueOutOfRange {
                value: num,
                min: 1,
                max: 2,
            });
        }
        self.num_channels = num;
        Ok(())
    }
    {% endif %}
    /// Query the name of the instrument
    ///
    /// TODO: Describe what this actually returns. Usually the simplest command to implement /
    /// start with
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        todo!();
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        todo!();
    }

    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        todo!();
    }
    {% if units %}
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
        todo!();
    }

    /// Update the unit by querying the instrument for the current unit setting.
    pub fn update_unit(&mut self) -> Result<(), InstrumentError> {
        todo!();
    }
{% endif -%}
}

{% if noc > 1 -%}
/// Channel structure representing a single channel of the {{ device }}.
///
/// **This structure can only be created through the [`{{ device | upper_camel_case }}`] struct.**
///
/// Implementation of an individual channel and commands that go to it.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
    {% if units -%}
    unit: Arc<Mutex<Unit>>,
    {% endif -%}
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the [`{{ device | upper_camel_case }}`] struct.
    fn new(idx: usize, interface: Arc<Mutex<T>>{% if units %}, unit: Arc<Mutex<Unit>>{% endif %}) -> Self {
        Channel {
            idx,
            interface,
            {% if units -%}
            unit,
            {% endif -%}
        }
    }

    /// Send a command for this instrument to an interface.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        todo!();
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        todo!();
    }
}
{% endif -%}
