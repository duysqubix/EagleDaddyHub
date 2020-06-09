mod api;
mod atcommands;
mod prelude;

use api::{
    RecieveApiFrame, TransmitApiFrame, TransmitRequestFrame, TransmitRequestOptions, TransmitStatus,
};
use atcommands::{AtCommand, AtCommands};
use bytes::{BufMut, BytesMut};
use prelude::*;
use serialport::{self, prelude::*};
use std::io::Write;
use std::thread;
use std::time::Duration;

struct DigiMeshDevice {
    serial: Box<dyn SerialPort>,
    cmd_mode: bool,
    rx_buf: BytesMut,
    tx_buf: BytesMut,
    //addr_64: u64,
    //addr_16: u16,
    //node_id: u16,
    //fw_version: u32,
    //hw_version: u16,
    //role:
}

impl DigiMeshDevice {
    fn new() -> Result<Self> {
        let settings = SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(20000),
        };
        Ok(Self {
            serial: serialport::open_with_settings("/dev/ttyUSB0", &settings)?,
            cmd_mode: false,
            rx_buf: BytesMut::with_capacity(128),
            tx_buf: BytesMut::with_capacity(128),
        })
    }

    fn send<'a>(&mut self, data: &'a [u8]) -> Result<usize> {
        Ok(self.serial.write(data)?)
    }

    fn recieve_atresponse(&mut self, expected_cr: usize) -> Result<()> {
        let mut buf: [u8; 1] = [0; 1];
        let mut cr_counter = 0;
        loop {
            if buf[0] == b'\r' {
                cr_counter += 1;
                if cr_counter == expected_cr {
                    break;
                }
            }
            self.serial.read_exact(&mut buf)?;
            self.rx_buf.put_u8(buf[0]);
        }

        if self.rx_buf.len() < 1 {
            return Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "RX buf empty",
            )));
        }
        Ok(())
    }
    /// send an AT command and returns the result
    fn atcmd<'a>(&mut self, atcmd: &'a AtCommand) -> Result<()> {
        let mut apply_changes = false;
        self.tx_buf.clear();
        self.rx_buf.clear();

        if atcmd.command != "+++" {
            self.tx_buf.put(&b"AT"[..]);
            self.tx_buf.put(atcmd.command.as_bytes());

            if let Some(data) = &atcmd.parameter {
                self.tx_buf.put(&data[..]);
                apply_changes = true;
            }
            self.tx_buf.put_u8(0x0d);
        } else {
            self.tx_buf.put(atcmd.command.as_bytes());
        }
        // we have constructed the AT commands, now just send it
        //println!("Sending: {:x?}", &tx_buf[..]);

        self.serial.write(&self.tx_buf[..])?;
        self.recieve_atresponse(atcmd.rcr_len)?;
        Ok(())
    }

    fn command_mode(&mut self, mode: bool) -> Result<()> {
        match mode {
            true => {
                thread::sleep(Duration::from_millis(1000));
                self.atcmd(&AtCommands::CmdMode(true).create())?;
                thread::sleep(Duration::from_millis(1000));
                self.cmd_mode = true;
            }
            false => {
                self.atcmd(&AtCommands::CmdMode(false).create())?;
                self.cmd_mode = false;
            }
        }
        Ok(())
    }

    fn apply_changes(&mut self) -> Result<()> {
        if self.cmd_mode == false {
            self.command_mode(true)?;
            return Err(Error::InvalidMode("Not in command mode".to_string()));
        }

        self.atcmd(&AtCommands::AtCmd(("AC", None)).create())?;

        self.atcmd(&AtCommands::AtCmd(("WR", None)).create())?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut device = DigiMeshDevice::new()?;
    // set api to 1
    device.command_mode(true)?;
    println!("{:x?}", device.rx_buf);
    device.atcmd(&AtCommands::AtCmd(("AP", Some(b"1"))).create())?;
    println!("{:x?}", device.rx_buf);
    device.command_mode(false)?;
    println!("{:x?}", device.rx_buf);
    println!("Attempting to send something");

    let packet = TransmitRequestFrame {
        dest_addr: api::BROADCAST_ADDR,
        broadcast_radius: 0,
        options: Some(&TransmitRequestOptions {
            disable_ack: false,
            disable_route_discovery: false,
            enable_unicast_nack: false,
            enable_unicast_trace_route: false,
            mode: api::MessagingMode::DigiMesh,
        }),
        payload: b"hi",
    }
    .gen()?;

    device.send(&packet[..])?;
    let mut status = TransmitStatus::default();
    status.recieve(device.serial.try_clone()?);
    println!("{:x?}", &packet[..]);
    println!("{:x?}", status);
    Ok(())
}
