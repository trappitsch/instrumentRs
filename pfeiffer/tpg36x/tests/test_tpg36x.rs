//! Tests for the Pfeiffer TPG36x driver.

use std::net::Ipv4Addr;

use rstest::*;

use instrumentrs::LoopbackInterface;

use pfeiffer_tpg36x::{DhcpConfig, EthernetConfig, Tpg36x};

type Tpg36Lbk = Tpg36x<LoopbackInterface<String>>;

const ENQ: &str = "\u{5}";
const ACK: &str = "\u{6}";

/// Function that takes input, output `Vec<&str>` and prepares the TPG36x instrument with this loopback
/// interface.
///
/// Note that it will automatically fill the input and output vectors with the unit query that is
/// performed when creating a new instrument instance. The unit is by default set to "Pa".
/// Furthermore, we will add the terminator to every command from (`host2inst` and `inst2host`),
/// except for the `ENQ`.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> Tpg36Lbk {
    let term = "\r\n";
    let mut inp = vec![format!("UNI{term}"), ENQ.to_string()];
    let mut out = vec![format!("{ACK}{term}"), format!("2{term}")];
    host2inst.iter().for_each(|s| {
        if *s != ENQ {
            inp.push(format!("{s}{term}"));
        } else {
            inp.push(s.to_string());
        }
    });
    inst2host
        .iter()
        .for_each(|s| out.push(format!("{s}{term}")));

    // initialize the interface with empty terminator, as we set it manually above!
    let interface = LoopbackInterface::new(inp, out, "");
    Tpg36x::try_new(interface).unwrap()
}

/// A fixture to create an empty TPG36x loopback interface.
#[fixture]
fn emp_tpg36x() -> Tpg36x<LoopbackInterface<String>> {
    crt_inst(vec![], vec![])
}

/// Ensure initialization of the instrument works correctly.
#[rstest]
fn test_initialization(_emp_tpg36x: Tpg36Lbk) {}

/// By default, instrument is set for TPG362, but can be configured as TPG361
#[rstest]
fn test_get_channel(mut emp_tpg36x: Tpg36Lbk) {
    assert!(emp_tpg36x.get_channel(0).is_ok());
    assert!(emp_tpg36x.get_channel(1).is_ok());
    assert!(emp_tpg36x.get_channel(2).is_err());

    // switch to one channel device
    emp_tpg36x.set_num_channels(1).unwrap();
    assert!(emp_tpg36x.get_channel(0).is_ok());
    assert!(emp_tpg36x.get_channel(1).is_err());

    // Only one and two channel instruments exist!
    assert!(emp_tpg36x.set_num_channels(3).is_err());
}

/// Set/get the ethernet configuration.
#[rstest]
fn test_ethernet_config() {
    // set to dynamic configuration
    let mut inst = crt_inst(
        vec![
            "ETH,1",
            "ETH",
            ENQ,
            "ETH,0,10.11.12.13,20.30.40.50,60.70.80.90",
            "ETH",
            ENQ,
        ],
        vec![
            ACK,
            ACK,
            "1,192.168.1.10,255.255.255.0,192.168.1.1",
            ACK,
            ACK,
            "0,10.11.12.13,20.30.40.50,60.70.80.90",
        ],
    );
    let dynamic_conf = EthernetConfig::new_dynamic();
    inst.set_ethernet_config(dynamic_conf).unwrap();

    // get the configuration and check it
    let conf1 = inst.get_ethernet_config().unwrap();
    assert_eq!(DhcpConfig::Dynamic, conf1.dhcp_conf);
    assert_eq!(Ipv4Addr::new(192, 168, 1, 10), conf1.ip.unwrap());
    assert_eq!(Ipv4Addr::new(255, 255, 255, 0), conf1.subnet_mask.unwrap());
    assert_eq!(Ipv4Addr::new(192, 168, 1, 1), conf1.gateway.unwrap());

    // set to static configuration
    let static_conf = EthernetConfig::new_static(
        Ipv4Addr::new(10, 11, 12, 13),
        Ipv4Addr::new(20, 30, 40, 50),
        Ipv4Addr::new(60, 70, 80, 90),
    );
    inst.set_ethernet_config(static_conf).unwrap();

    let conf2 = inst.get_ethernet_config().unwrap();
    assert_eq!(DhcpConfig::Static, conf2.dhcp_conf);
    assert_eq!(Ipv4Addr::new(10, 11, 12, 13), conf2.ip.unwrap());
    assert_eq!(Ipv4Addr::new(20, 30, 40, 50), conf2.subnet_mask.unwrap());
    assert_eq!(Ipv4Addr::new(60, 70, 80, 90), conf2.gateway.unwrap());
}

/// Get the name of the unit.
#[rstest]
fn test_get_name() {
    let mut inst = crt_inst(vec!["AYT", ENQ], vec![ACK, "ASDF1234"]);
    assert_eq!("ASDF1234", inst.get_name().unwrap());
}
