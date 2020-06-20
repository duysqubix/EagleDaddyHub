#![deny(missing_docs)]

use crate::modules::{self, Module};
use bytes::BytesMut;
use rustbee::{
    api::{self, RecieveApiFrame},
    device::{self, DigiMeshDevice},
};
use serde_yaml;
use std::error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub enum Error {
    /// NoDetectedModules
    ///
    /// There were no valid uids detected.
    NoDetectedModules,

    /// XBee Device error
    DeviceError(device::Error),

    /// XBee Api Error
    ApiError(api::Error),

    ///IO Error
    IOError(std::io::Error),

    /// Module Error
    ModuleError(modules::Error),
    InvalidResponse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::NoDetectedModules => write!(f, "No valid modules detected"),
            Error::ApiError(ref err) => write!(f, "{}", err),
            Error::DeviceError(ref err) => write!(f, "{}", err),
            Error::IOError(ref err) => write!(f, "{}", err),
            Error::ModuleError(ref err) => write!(f, "{}", err),
            Error::InvalidResponse(ref err) => write!(f, "{}", err),
        }
    }
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<modules::Error> for Error {
    fn from(err: modules::Error) -> Error {
        Error::ModuleError(err)
    }
}

impl error::Error for Error {}

/// alias for encapsulation of result for manager
pub type Result<T> = std::result::Result<T, Error>;

pub enum ModuleCommands {
    RequestTime,
    RequestTH,
    RequestDist,
    RequestMotor,
    InvalidCmd,
}

impl ModuleCommands {
    pub fn value(&self) -> u8 {
        match *self {
            ModuleCommands::RequestTH => 0x2b,
            ModuleCommands::RequestTime => 0x1d,
            ModuleCommands::RequestDist => 0x3c,
            ModuleCommands::RequestMotor => 0x4a,
            ModuleCommands::InvalidCmd => 0x00,
        }
    }
}

/// manages the different slave modules
#[derive(Debug)]
pub struct ModuleManager {
    /// holds the main XBee device connected to computer
    pub device: DigiMeshDevice,

    /// holds the different statically defined modules by the trait: DigitalComponent
    pub modules: Vec<Module>,
}

impl ModuleManager {
    /// creates an instance of a module manager
    pub fn new<'a>(port: &'a str, baud: u32) -> Result<Self> {
        Ok(Self {
            device: DigiMeshDevice::new(port, baud)?,
            modules: Vec::new(),
        })
    }

    /// Saves modules to disk
    pub fn dump_to_disk(&mut self) -> Result<()> {
        let s = serde_yaml::to_string(&self.modules).unwrap();
        let mut f = File::create(".modules")?;
        f.write_all(&s[..].as_bytes())?;
        Ok(())
    }

    pub fn load_modules(&mut self) -> Result<()> {
        let mut f = File::open(".modules")?;

        let mut modules = String::new();
        f.read_to_string(&mut modules)?;
        let module_vec: Vec<Module> = serde_yaml::from_str(&modules).unwrap();

        self.modules = module_vec;

        Ok(())
    }

    // get mutable reference of Module
    pub fn get_module<'a>(&mut self, node_id: &'a str) -> Option<usize> {
        let mut idx = None;
        for (i, m) in self.modules.iter().enumerate() {
            if m.device.node_id == String::from(node_id).to_ascii_uppercase() {
                idx = Some(i);
                break;
            }
        }

        idx
    }

    // gets list of node_id from modules in list
    pub fn get_node_ids(&self) -> Result<Vec<String>> {
        let mut v: Vec<String> = Vec::new();
        for m in self.modules.iter() {
            v.push(m.device.node_id.clone());
        }

        Ok(v)
    }

    // sent transmit_request, expect RecieveFrame
    pub fn transmit_request(
        &mut self,
        request: api::TransmitRequestFrame,
    ) -> Result<api::RecieveRequestFrame> {
        let _ = self.device.send_frame(request)?;
        let response =
            api::RecieveRequestFrame::recieve(self.device.serial.try_clone().unwrap()).unwrap();

        Ok(response)
    }

    pub fn request(&mut self, module_idx: usize, cmd: ModuleCommands) -> Result<()> {
        // request temp and humditity
        let m = self.modules.get(module_idx).unwrap(); // this should be safe...

        let mut payload: BytesMut = BytesMut::from(&vec![0 as u8; 8][..]);
        payload[0] = (m.id >> 8) as u8;
        payload[1] = (m.id as u8) & 0xff;
        payload[2] = cmd.value();

        let transmit_request = api::TransmitRequestFrame {
            dest_addr: m.device.addr_64bit,
            options: None,
            broadcast_radius: 0,
            payload: &payload[..],
        };

        let response = self.transmit_request(transmit_request)?;
        // check to make sure response is correct length
        if response.rf_data.len() < 4 {
            return Err(Error::InvalidResponse(format!(
                "RF data length is wrong: {}",
                response.rf_data.len()
            )));
        }

        if response.rf_data[2] == 0xff {
            // device returned error, now we can return error code
            let err_code = response.rf_data[3];
            return Err(Error::DeviceError(device::Error::RemoteDeviceError(
                format!("device returned error code: {}", err_code),
            )));
        }
        println!("{:x?}:{:x?}", &response.dest_addr, &response.rf_data[..]);
        Ok(())
    }

    /// discovers nodes on the network by querying the module_id of each node in range
    pub fn discovery_mode(&mut self) -> Result<()> {
        let broadcast_id = api::TransmitRequestFrame {
            dest_addr: api::BROADCAST_ADDR,
            broadcast_radius: 0,
            options: None,
            payload: b"\x0a\xaa",
        };

        let _transmit_status = self.device.send_frame(broadcast_id)?;
        //println!("{:#x?}", transmit_status);

        loop {
            let reply = api::RecieveRequestFrame::recieve(
                self.device
                    .serial
                    .try_clone()
                    .expect("Could not clone serial"),
            );
            match reply {
                Ok(resp) => {
                    println!("Recieve Status: 0x{:02x}", resp.recv_options);
                    let module_id = ((resp.rf_data[0] as u16) << 8) | (resp.rf_data[1] as u16);

                    let module = Module {
                        id: module_id,
                        device: device::RemoteDigiMeshDevice {
                            addr_64bit: resp.dest_addr,
                            node_id: "NotSet".to_string(),
                            firmware_version: None,
                            hardware_version: None,
                        },
                    };

                    if self.modules.contains(&module) == false {
                        self.modules.push(module);
                    }
                }
                Err(_) => break,
            }
        }

        // now query information for each module
        for module in self.modules.iter_mut() {
            module.get_info(&mut self.device)?;
        }
        Ok(())
    }
}
