mod api;
mod atcommands;
mod prelude;

use api::ApiFrame;
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

        let mut mini_buf: [u8; 1] = [0; 1];
        let mut cr_counter = 0;
        loop {
            if mini_buf[0] == b'\r' {
                cr_counter += 1;

                if cr_counter == atcmd.rcr_len {
                    break;
                }
            }
            self.serial.read_exact(&mut mini_buf)?;
            self.rx_buf.put_u8(mini_buf[0]);
        }

        if self.rx_buf.len() < 1 {
            return Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "RX buf empty",
            )));
        }

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
            self.command_mode(true);
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
    device.atcmd(&AtCommands::CmdMode(true).create())?;
    println!("{:x?}", device.rx_buf);
    device.atcmd(&AtCommands::AtCmd(("AP", Some(b"1"))).create())?;
    println!("{:x?}", device.rx_buf);
    device.apply_changes();
    device.command_mode(false)?;
    println!("{:x?}", device.rx_buf);
    println!("Attempting to send something");

    let packet = device
        .api
        .transmit_request(api::BROADCAST_ADDR, 0, 0, b"HELLO FROM API")?;
    device.serial.write(&packet[..]);
    println!("{:x?}", &packet[..]);
    // let remote_name = b"MYREMOTE";
    // let mut device = DigiMeshDevice::new()?;
    // let test = AtCommands::AtCmd(("IP", None));
    // let test2 = AtCommands::AtCmd(("AP", None));
    // let rename_node = AtCommands::AtCmd(("NI", Some(remote_name)));

    // let discover = AtCommands::Discover(None);
    // device.command_mode(true)?;
    // println!("{:?}: {:?}", test, device.atcmd(&test.create())?);
    // println!("{:?}: {:?}", test2, device.atcmd(&test2.create())?);
    // println!(
    //     "{:?}: {:?}",
    //     rename_node,
    //     device.atcmd(&rename_node.create())?
    // );
    // println!("{:?}: {:?}", discover, device.atcmd(&discover.create())?);
    // device.command_mode(false)?;
    Ok(())
}
