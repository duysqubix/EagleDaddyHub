#![deny(missing_docs)]

use crate::modules::Module;
use crate::prelude::*;
use rustbee::{
    api::{self, RecieveApiFrame},
    device::{self, DigiMeshDevice, RemoteDigiMeshDevice},
};
use std::error;

#[derive(Debug)]
pub enum Error {
    /// NoDetectedModules
    ///
    /// There were no valid uids detected.
    NoDetectedModules,
    /// Unknown UID
    ///
    /// Recieved an unknown UID from slave device
    UnknownUID(Uid),
    /// XBee Device error
    DeviceError(device::Error),
    /// XBee Api Error
    ApiError(api::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::NoDetectedModules => write!(f, "No valid modules detected"),
            Error::UnknownUID(ref uid) => write!(f, "Unknown UID: 0x{:x?}", uid),
            Error::ApiError(ref err) => write!(f, "{}", err),
            Error::DeviceError(ref err) => write!(f, "{}", err),
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

impl error::Error for Error {}

/// alias for encapsulation of result for manager
pub type Result<T> = std::result::Result<T, Error>;

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
    // TODO once in discovery mode completes, take nodes and store them persistently
    // In fact, don't use built in Discover mode on XBee, write custom firmware that waits for
    // command to identify itself, generating a transmit request frame with appropriate payload!
    pub fn discovery_mode(&mut self) -> Result<()> {
        let broadcast_id = api::TransmitRequestFrame {
            dest_addr: api::BROADCAST_ADDR,
            broadcast_radius: 0,
            options: None,
            payload: b"\x0a\xaa",
        };

        let transmit_status = self.device.send_frame(broadcast_id)?;
        println!("{:#x?}", transmit_status);

        let reply = api::RecieveRequestFrame::recieve(self.device.serial.try_clone().unwrap())?;
        println!("{:#x?}", reply);

        Ok(())
    }
}
