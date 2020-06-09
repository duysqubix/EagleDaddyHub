#![allow(dead_code)]
//!
//! XBee API Frame
//!
//!

use bytes::{BufMut, BytesMut};
use rand::{thread_rng, Rng};
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

pub enum ResponseType {
    TransmitStatus,
}

pub trait RecieveApiFrame {
    fn recieve(&mut self, ser: Box<dyn SerialPort>) -> Result<()>;
    fn delim(&self) -> u8 {
        0x7e
    }
}

#[derive(Debug)]
pub struct TransmitStatus {
    frame_id: u8,
    transmit_retry_count: u8,
    deliver_status: u8,
    discovery_status: u8,
}

impl TransmitStatus {
    pub fn default() -> Self {
        Self {
            frame_id: 0,
            transmit_retry_count: 0,
            deliver_status: 0,
            discovery_status: 0,
        }
    }
}

impl RecieveApiFrame for TransmitStatus {
    fn recieve(&mut self, mut ser: Box<dyn SerialPort>) -> Result<()> {
        // wait for first
        let mut header: [u8; 3] = [0; 3];
        loop {
            ser.read(&mut header)?;
            if header[0] == self.delim() {
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
        ser.read(&mut payload[..]);
        println!("payload: {:x?}", payload);
        self.frame_id = payload[1];
        self.transmit_retry_count = payload[4];
        self.deliver_status = payload[5];
        self.discovery_status = payload[6];
        Ok(())
    }
}

pub trait TransmitApiFrame {
    fn gen(&self) -> Result<BytesMut>;
    fn delim(&self) -> u8 {
        0x7e
    }
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
