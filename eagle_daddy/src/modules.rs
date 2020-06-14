//!
//!
//! Meta Information on support for different Modules
//!
//!

use rustbee::{api, device};
use std::collections::HashMap;

static MODULE_PROTOTYPE: u16 = 0x001a;
static MODULE_DEERFEEDER: u16 = 0x002b;

#[derive(Debug)]
pub enum Error {
    NotValidModule,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::NotValidModule => write!(f, "Not a valid module option"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Module<'a> {
    id: u16,
    cmd_map: HashMap<&'a str, u8>,
}

impl Module<'_> {
    pub fn new<'a>(m: &'a str) -> Result<Self> {
        let mut cmd_map: HashMap<&'static str, u8> = HashMap::new();
        match m {
            "prototype" => {
                // do something here
                cmd_map.insert("RequestMotorTime", 0x4a);
                cmd_map.insert("RequestTime", 0x1d);
                cmd_map.insert("RequestTempH", 0x2b);
                cmd_map.insert("RequestDistance", 0x3c);

                Ok(Self {
                    id: 0x001a,
                    cmd_map: cmd_map,
                })
            }
            _ => Err(Error::NotValidModule),
        }
    }

    pub fn send_cmd<'a>(
        host: &'a mut device::DigiMeshDevice,
        cmd: &'a str,
        data: Option<&'a [u8]>,
    ) {
        // contruct payload
        let transmit_status = api::TransmitRequestFrame{
            dest_addr: api::BROADCAST_ADDR
        }
    }
}
//let module = Module::new("prototype");
//let response = module.send_cmd("RequestMotorTime", Some(b"15"))?;
