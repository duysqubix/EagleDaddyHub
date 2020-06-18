//!
//!
//! Meta Information on support for different Modules
//!
//!

use rustbee::{api, device};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub id: u16,
    pub device: device::RemoteDigiMeshDevice,
}

impl Module {}
