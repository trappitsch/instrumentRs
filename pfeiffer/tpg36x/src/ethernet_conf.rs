//! Ethernet configuration for the TPG36x.

use std::{fmt::Display, net::Ipv4Addr};

use instrumentrs::InstrumentError;

/// An enum for the DHCP configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DhcpConfig {
    /// Static DHCP configuration
    Static,
    /// Dynamic DHCP configuration
    Dynamic,
}

impl DhcpConfig {
    /// Convert the `DhcpConfig` to a string representation.
    fn as_str(&self) -> &str {
        match self {
            DhcpConfig::Static => "0",
            DhcpConfig::Dynamic => "1",
        }
    }
}

impl Display for DhcpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DhcpConfig::Static => write!(f, "Static"),
            DhcpConfig::Dynamic => write!(f, "Dynamic"),
        }
    }
}

impl TryFrom<&str> for DhcpConfig {
    type Error = InstrumentError;

    /// Convert a string that is received from the device to a `DhcpConfig`.
    ///
    /// - String: "0" -> static configuration
    /// - String: "1" -> dynamic configuration
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "0" => Ok(DhcpConfig::Static),
            "1" => Ok(DhcpConfig::Dynamic),
            _ => Err(InstrumentError::ResponseParseError(value.to_string())),
        }
    }
}

/// Ethernet configuration for the TPG36x.
///
/// All IPs must be defined as IPv4 addresses, as this is the only supported protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthernetConfig {
    /// The DHCP configuration.
    pub dhcp_conf: DhcpConfig,
    /// IP address.
    pub ip: Option<Ipv4Addr>,
    /// Subnet mask address.
    pub subnet_mask: Option<Ipv4Addr>,
    /// Gateway address.
    pub gateway: Option<Ipv4Addr>,
}

impl EthernetConfig {
    /// Create a new dynamic DHCP configuration to send to the TPG36x.
    pub fn new_dynamic() -> Self {
        EthernetConfig {
            dhcp_conf: DhcpConfig::Dynamic,
            ip: None,
            subnet_mask: None,
            gateway: None,
        }
    }

    /// Create a new static configuration to send to the TPG36x.
    ///
    /// # Arguments:
    /// - `ip`: The IP address to set for the TPG36x.
    /// - `subnet_mask`: The subnet mask to set for the TPG36x.
    /// - `gateway`: The gateway to set for the TPG36x.
    pub fn new_static(ip: Ipv4Addr, subnet_mask: Ipv4Addr, gateway: Ipv4Addr) -> Self {
        EthernetConfig {
            dhcp_conf: DhcpConfig::Static,
            ip: Some(ip),
            subnet_mask: Some(subnet_mask),
            gateway: Some(gateway),
        }
    }

    /// Convert a string that is received from the device to an `EthernetConfig`.
    ///
    /// If this fails, it will return an `InstrumentError::ResponseParseError`, since the only
    /// failure scenario should be a malformed response from the device.
    pub(crate) fn from_cmd_str(value: &str) -> Result<Self, InstrumentError> {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 4 {
            return Err(InstrumentError::ResponseParseError(value.to_string()));
        }
        let ip = parts[1]
            .parse::<Ipv4Addr>()
            .map_err(|_| InstrumentError::ResponseParseError(value.to_string()))?;
        let subnet_mask = parts[2]
            .parse::<Ipv4Addr>()
            .map_err(|_| InstrumentError::ResponseParseError(value.to_string()))?;
        let gateway = parts[3]
            .parse::<Ipv4Addr>()
            .map_err(|_| InstrumentError::ResponseParseError(value.to_string()))?;
        let dhcp_conf = DhcpConfig::try_from(parts[0])?;
        match dhcp_conf {
            DhcpConfig::Dynamic => Ok(EthernetConfig {
                dhcp_conf: DhcpConfig::Dynamic,
                ip: Some(ip),
                subnet_mask: Some(subnet_mask),
                gateway: Some(gateway),
            }),
            DhcpConfig::Static => Ok(EthernetConfig::new_static(ip, subnet_mask, gateway)),
        }
    }

    /// Turn the Ethernet configuration into a command string that can be sent to the TPG36x.
    pub(crate) fn to_command_string(&self) -> String {
        match self.dhcp_conf {
            DhcpConfig::Dynamic => format!("ETH,{}", self.dhcp_conf.as_str()),
            DhcpConfig::Static => format!(
                "ETH,{},{},{},{}",
                self.dhcp_conf.as_str(),
                self.ip.expect("This should be infallible."),
                self.subnet_mask.expect("This should be infallible."),
                self.gateway.expect("This should be infallible.")
            ),
        }
    }
}

impl Display for EthernetConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret_str = format!("DHCP config: {}\n", self.dhcp_conf);
        if let Some(ip) = self.ip {
            ret_str.push_str("IP Adress: ");
            ret_str.push_str(&ip.to_string());
            ret_str.push('\n');
        }
        if let Some(subnet) = self.subnet_mask {
            ret_str.push_str("Subnet mask: ");
            ret_str.push_str(&subnet.to_string());
            ret_str.push('\n');
        }
        if let Some(gateway) = self.gateway {
            ret_str.push_str("Gateway: ");
            ret_str.push_str(&gateway.to_string());
        }
        write!(f, "{ret_str}")
    }
}
