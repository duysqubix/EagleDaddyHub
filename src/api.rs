//!
//! XBee API Frame
//!
//!

use bytes::{BufMut, BytesMut};
use rand::{thread_rng, Rng};

static DELIM: u8 = 0x7e;
pub static BROADCAST_ADDR: u64 = 0xffff;
#[derive(Debug)]
pub enum Error {
    FrameError(String),
    PayloadError(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::FrameError(ref err) => write!(f, "{}", err),
            Error::PayloadError(ref err) => write!(f, "{}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

//device.api.transmit_request(BROADCAST_ADDR, b"HELLO WORLD")?;
//device.api

pub struct ApiFrame {
    delim: u8,
    length: u16,
    checksum: u8,
}

impl<'a> ApiFrame {
    pub fn new() -> Self {
        Self {
            delim: 0x7e,
            length: 0,
            checksum: 0,
        }
    }

    pub fn transmit_request(
        &mut self,
        dest_addr: u64,
        broad_cast_radius: u8,
        options: u8,
        payload: &'a [u8],
    ) -> Result<BytesMut> {
        let mut packet = BytesMut::new();
        let mut rng = rand::thread_rng();

        if payload.len() > 65535 - 112 {
            return Err(Error::PayloadError(
                "Payload exceeds maximum size".to_string(),
            ));
        }
        let frame_id: u8 = rng.gen();
        packet.put_u8(DELIM);
        packet.put_u16((payload.len() as u16) + (0x0e as u16));
        packet.put_u8(0x10);
        packet.put_u8(0x01);
        packet.put_u64(dest_addr);
        packet.put_u16(0xfffe);
        packet.put_u8(broad_cast_radius);
        packet.put_u8(options);
        packet.put(payload);

        let chksum = self.calc_checksum(&packet[..])?;
        packet.put_u8(chksum);

        Ok(packet)
    }

    pub fn calc_checksum(&self, frame: &[u8]) -> Result<u8> {
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

    pub fn verify_checksum(&self, frame: &[u8]) -> Result<bool> {
        let data_wo_chksum = &frame[..frame.len() - 2]; // exclude checksum;
        let provided_chksum = frame[frame.len() - 1];

        let calc_chksum = self.calc_checksum(data_wo_chksum)?;

        if provided_chksum != calc_chksum {
            return Ok(false);
        }
        Ok(true)
    }
}
