#![deny(missing_docs)]

use crate::modules::{self, Module};
use bytes::BytesMut;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
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
    SetTime,
    SetSchedule,
    SetMotorTime,
    InvalidCmd,
}

impl ModuleCommands {
    pub fn value(&self) -> u8 {
        match *self {
            ModuleCommands::RequestTH => 0x2b,    // returns temp and humidity
            ModuleCommands::RequestTime => 0x1d,  // returns current time from RTC
            ModuleCommands::RequestDist => 0x3c,  // returns distance in cm
            ModuleCommands::RequestMotor => 0x4a, // returns motor time in s (how long it should stay on)
            ModuleCommands::InvalidCmd => 0x00,   // debug
            ModuleCommands::SetTime => 0x5e,      // sets RTC with current module
            ModuleCommands::SetSchedule => 0x6c,  // sets the scheduling when to turn on motor
            ModuleCommands::SetMotorTime => 0x7a, // sets how long the motor is on:w
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
        let response = api::RecieveRequestFrame::recieve(self.device.serial.try_clone().unwrap())?;

        Ok(response)
    }

    pub fn set(
        &mut self,
        module_idx: usize,
        cmd: ModuleCommands,
        args: Option<&Vec<String>>,
    ) -> Result<api::RecieveRequestFrame> {
        let m = self.modules.get(module_idx).unwrap();

        let mut payload: BytesMut = BytesMut::from(&vec![0 as u8; 16][..]);
        payload[0] = (m.id >> 8) as u8;
        payload[1] = (m.id as u8) & 0xff;
        payload[2] = cmd.value();

        match cmd {
            ModuleCommands::SetTime => {
                let dt: DateTime<Local> = Local::now();

                let seconds = dt.second() as u8;
                let minute = dt.minute() as u8;
                let hour = dt.hour() as u8;
                let day = dt.day() as u8;
                let month = dt.month() as u8;
                let year = dt.year() as u16;

                payload[3] = seconds;
                payload[4] = minute;
                payload[5] = hour;
                payload[6] = day;
                payload[7] = month;
                payload[8] = (year >> 8) as u8;
                payload[9] = (year & 0xff) as u8;
                Ok(())
            }
            ModuleCommands::SetSchedule => {
                // helelo
                payload[3] = 9;
                payload[4] = 00;
                payload[5] = 12;
                payload[6] = 00;
                payload[7] = 15;
                payload[8] = 00;
                payload[9] = 18;
                payload[10] = 00;

                Ok(())
            }
            ModuleCommands::SetMotorTime => {
                if let Some(args) = args {
                    if args.len() < 4 {
                        return Err(Error::NoDetectedModules);
                    }
                    payload[3] = args[3].parse::<u8>().unwrap();
                } else {
                    payload[3] = 3;
                }
                Ok(())
            }
            _ => Err(Error::NoDetectedModules),
        }?;

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
        //println!("{:x?}:{:x?}", &response.dest_addr, &response.rf_data[..]);
        Ok(response)
    }

    pub fn request(
        &mut self,
        module_idx: usize,
        cmd: ModuleCommands,
    ) -> Result<api::RecieveRequestFrame> {
        let m = self.modules.get(module_idx).unwrap(); // this should be safe...

        let mut payload: BytesMut = BytesMut::from(&vec![0 as u8; 16][..]);
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
        //println!("{:x?}:{:x?}", &response.dest_addr, &response.rf_data[..]);
        Ok(response)
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
                    println!(
                        "Responding [{:16x}:0x{:02x}]",
                        resp.dest_addr, resp.recv_options
                    );
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
