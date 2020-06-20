//!
//!
//! Meta Information on support for different Modules
//!
//!

use rustbee::{api, device};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Error {
    DeviceError(device::Error),
    ApiError(api::Error),
}

impl From<device::Error> for Error {
    fn from(err: device::Error) -> Error {
        Error::DeviceError(err)
    }
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Error {
        Error::ApiError(err)
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::DeviceError(ref err) => write!(f, "{}", err),
            Error::ApiError(ref err) => write!(f, "{}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub id: u16,
    pub device: device::RemoteDigiMeshDevice,
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.device.addr_64bit.partial_cmp(&other.device.addr_64bit)
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.device.addr_64bit == other.device.addr_64bit
    }
}
impl Module {
    pub fn get_info(&mut self, master: &mut device::DigiMeshDevice) -> Result<()> {
        // ask for node_id
        let mut attempt = false;

        print!("Query ID: {:x?} => ", self.device.addr_64bit);
        let atresponse = master.send_frame(api::RemoteAtCommandFrame {
            dest_addr: self.device.addr_64bit,
            options: &api::RemoteCommandOptions {
                apply_changes: false,
            },
            atcmd: "NI",
            cmd_param: None,
        })?;

        if let Some(resp) = atresponse.downcast_ref::<api::RemoteAtCommandResponse>() {
            if let Some(ref cmd_data) = resp.command_data {
                let id = std::str::from_utf8(&cmd_data[..]);
                if let Ok(id) = id {
                    self.device.node_id = String::from(id);
                    attempt = true;
                }
            }
        }
        println!("{}", attempt);
        attempt = false;

        print!("Query HW: {:x?} => ", self.device.addr_64bit);
        let atresponse = master.send_frame(api::RemoteAtCommandFrame {
            dest_addr: self.device.addr_64bit,
            options: &api::RemoteCommandOptions {
                apply_changes: false,
            },
            atcmd: "HV",
            cmd_param: None,
        })?;
        if let Some(resp) = atresponse.downcast_ref::<api::RemoteAtCommandResponse>() {
            if let Some(ref cmd_data) = resp.command_data {
                let hv = u16::from_be_bytes(<[u8; 2]>::try_from(&cmd_data[..]).unwrap());
                self.device.hardware_version = Some(hv);
                attempt = true;
            }
        }
        println!("{}", attempt);
        attempt = false;

        print!("Query VR: {:x?} => ", self.device.addr_64bit);
        let atresponse = master.send_frame(api::RemoteAtCommandFrame {
            dest_addr: self.device.addr_64bit,
            options: &api::RemoteCommandOptions {
                apply_changes: false,
            },
            atcmd: "VR",
            cmd_param: None,
        })?;
        if let Some(resp) = atresponse.downcast_ref::<api::RemoteAtCommandResponse>() {
            if let Some(ref cmd_data) = resp.command_data {
                let vr = u16::from_be_bytes(<[u8; 2]>::try_from(&cmd_data[..]).unwrap());
                self.device.firmware_version = Some(vr);
                attempt = true;
            }
        }
        println!("{}", attempt);
        Ok(())
    }
}
