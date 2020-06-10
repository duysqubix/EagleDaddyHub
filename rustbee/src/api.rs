#![allow(dead_code)]
//!
//! XBee API Frame
//!
//!

use bytes::{BufMut, BytesMut};
use rand::Rng;
use serialport::prelude::*;

static DELIM: u8 = 0x7e;
pub static BROADCAST_ADDR: u64 = 0xffff;
#[derive(Debug)]
pub enum Error {
    FrameError(String),
    PayloadError(String),
    IOError(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::FrameError(ref err) => write!(f, "{}", err),
            Error::PayloadError(ref err) => write!(f, "{}", err),
            Error::IOError(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum FrameId {
    TransmitRequest,
    TransmitStatus,
    Null,
}

impl FrameId {
    fn id(&self) -> u8 {
        match *self {
            FrameId::TransmitRequest => 0x90,
            FrameId::TransmitStatus => 0x8b,
            FrameId::Null => 0xff,
        }
    }
}

pub trait RecieveApiFrame {
    fn recieve(ser: Box<dyn SerialPort>) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn id(&self) -> FrameId;
    fn summary(&self) -> ();
}

pub trait TransmitApiFrame {
    fn gen(&self) -> Result<BytesMut>;
    fn delim(&self) -> u8 {
        0x7e
    }
    fn id(&self) -> FrameId;
    fn calc_checksum(&self, frame: &[u8]) -> Result<u8> {
        if frame.len() < 5 {
            return Err(Error::FrameError(
                "Frame length does not meet minimum requirements".to_string(),
            ));
        }

        let mut checksum: u64 = 0;
        for (pos, byte) in frame.iter().enumerate() {
            if pos > 2 {
                checksum += *byte as u64;
            }
        }

        Ok(0xff - (checksum as u8))
    }
}

/**
 * AtCommand Support
 *
 *
 */
pub struct AtCommand<'a> {
    pub command: &'a str,
    pub parameter: &'a Option<&'a [u8]>,
    pub rcr_len: usize, // the number of carriage returns in the reponse for this command
}

#[derive(Debug)]
pub enum AtCommands<'a> {
    Discover(Option<&'a [u8]>),
    AtCmd((&'a str, Option<&'a [u8]>)),
    CmdMode(bool),
}

impl AtCommands<'_> {
    pub fn create(&self) -> AtCommand {
        match *self {
            AtCommands::CmdMode(ref state) => match state {
                true => AtCommand {
                    command: "+++",
                    parameter: &None,
                    rcr_len: 1,
                },
                false => AtCommand {
                    command: "CN",
                    parameter: &None,
                    rcr_len: 1,
                },
            },
            AtCommands::Discover(ref param) => AtCommand {
                command: "ND",
                parameter: param,
                rcr_len: 10 + 1,
            },
            AtCommands::AtCmd((ref cmd, ref param)) => AtCommand {
                command: cmd,
                parameter: param,
                rcr_len: 1,
            },
        }
    }
}
/**
 * /AtCommand Support  
 **/

#[derive(Debug)]
pub struct NullRecieve;
impl RecieveApiFrame for NullRecieve {
    fn id(&self) -> FrameId {
        FrameId::Null
    }
    fn recieve(mut _ser: Box<dyn SerialPort>) -> Result<Self> {
        Ok(Self)
    }

    fn summary(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
pub struct TransmitStatus {
    frame_id: u8,
    transmit_retry_count: u8,
    deliver_status: u8,
    discovery_status: u8,
}

impl RecieveApiFrame for TransmitStatus {
    fn summary(&self) -> () {
        println!("{:?}", self);
    }
    fn id(&self) -> FrameId {
        FrameId::TransmitStatus
    }

    fn recieve(mut ser: Box<dyn SerialPort>) -> Result<Self> {
        // wait for first
        let mut header: [u8; 3] = [0; 3];
        loop {
            ser.read(&mut header)?;
            if header[0] == DELIM {
                break;
            } else {
                return Err(Error::PayloadError(
                    "Start Delimiter not found in response packet".to_string(),
                ));
            }
        }
        let length: usize = ((header[1] as usize) << 8) | (header[0] as usize);
        println!("{}", length);
        let mut payload: Vec<u8> = vec![0; length + 1];
        ser.read(&mut payload[..])?;
        println!("payload: {:x?}", payload);
        Ok(Self {
            frame_id: payload[1],
            transmit_retry_count: payload[4],
            deliver_status: payload[5],
            discovery_status: payload[6],
        })
    }
}

pub enum MessagingMode {
    PointToPoint,
    Repeater,
    DigiMesh,
}

pub struct TransmitRequestOptions {
    pub disable_ack: bool,
    pub disable_route_discovery: bool,
    pub enable_unicast_nack: bool,
    pub enable_unicast_trace_route: bool,
    pub mode: MessagingMode,
}

impl TransmitRequestOptions {
    pub fn compile(&self) -> u8 {
        let mut val: u8 = 0;

        if self.disable_ack == true {
            val |= 1 << 0;
        }
        if self.disable_route_discovery == true {
            val |= 1 << 1;
        }
        if self.enable_unicast_nack == true {
            val |= 1 << 2;
        }

        if self.enable_unicast_trace_route == true {
            val |= 1 << 3;
        }

        match self.mode {
            MessagingMode::PointToPoint => (0x1 << 6) | val,
            MessagingMode::Repeater => (0x2 << 6) | val,
            MessagingMode::DigiMesh => (0x3 << 6) | val,
        }
    }
}

pub struct TransmitRequestFrame<'a> {
    pub dest_addr: u64,
    pub broadcast_radius: u8,
    pub options: Option<&'a TransmitRequestOptions>,
    pub payload: &'a [u8],
}

impl TransmitApiFrame for TransmitRequestFrame<'_> {
    fn id(&self) -> FrameId {
        FrameId::TransmitRequest
    }

    fn gen(&self) -> Result<BytesMut> {
        let mut packet = BytesMut::new();
        let mut rng = rand::thread_rng();
        if self.payload.len() > 65535 - 112 {
            return Err(Error::PayloadError("Payload exceeds max size".to_string()));
        }

        let frame_id: u8 = rng.gen();

        packet.put_u8(self.delim());
        packet.put_u16((self.payload.len() as u16) + (0x0e as u16));
        packet.put_u8(0x10);
        packet.put_u8(frame_id);
        packet.put_u64(self.dest_addr);
        packet.put_u16(0xfffe);
        packet.put_u8(self.broadcast_radius);

        match self.options {
            Some(opts) => packet.put_u8(opts.compile()),
            None => packet.put_u8(0),
        }
        packet.put(self.payload);

        let chksum = self.calc_checksum(&packet[..])?;
        packet.put_u8(chksum);

        Ok(packet)
    }
}
