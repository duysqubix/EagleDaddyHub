mod atcommands;
mod prelude;

use atcommands::{AtCommand, AtCommands};
use bytes::{BufMut, BytesMut};
use prelude::*;
use serialport::{self, prelude::*};
use std::io::Write;
use std::thread;
use std::time::Duration;
struct DigiMeshDevice {
    serial: Box<dyn SerialPort>,
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
        })
    }

    fn send<'a>(&mut self, data: &'a [u8]) -> Result<usize> {
        Ok(self.serial.write(data)?)
    }

    /// send an AT command and returns the result
    fn atcmd<'a>(&mut self, atcmd: &'a AtCommand) -> Result<BytesMut> {
        let mut tx_buf = BytesMut::with_capacity(500);
        let mut recv_buf = BytesMut::with_capacity(500);

        if atcmd.command != "+++" {
            tx_buf.put(&b"AT"[..]);
            tx_buf.put(atcmd.command.as_bytes());

            if let Some(data) = &atcmd.parameter {
                tx_buf.put(&data[..]);
            }
            tx_buf.put_u8(0x0d);
        } else {
            tx_buf.put(atcmd.command.as_bytes());
        }
        // we have constructed the AT commands, now just send it
        //println!("Sending: {:x?}", &tx_buf[..]);
        self.send(&tx_buf[..])?;

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
            // println!("{}", mini_buf[0]);
            //println!("{:x?}", mini_buf[0]);
            recv_buf.put_u8(mini_buf[0]);
        }
        Ok(recv_buf)
    }

    fn cmd_mode(&mut self, mode: bool) -> Result<()> {
        match mode {
            true => {
                thread::sleep(Duration::from_millis(1000));
                println!("{:?}", self.atcmd(&AtCommands::CmdOn.create())?);
                thread::sleep(Duration::from_millis(1000));
            }
            false => println!("{:?}", self.atcmd(&AtCommands::CN.create())?),
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let remote_name = b"MYREMOTE";
    let mut device = DigiMeshDevice::new()?;
    let test = AtCommands::ID(None);
    let test2 = AtCommands::AP(None);
    let rename_node = AtCommands::NI(Some(remote_name));

    let discover = AtCommands::ND(Some(remote_name));
    device.cmd_mode(true)?;
    println!("{:?}: {:?}", test, device.atcmd(&test.create())?);
    println!("{:?}: {:?}", test2, device.atcmd(&test2.create())?);
    println!(
        "{:?}: {:?}",
        rename_node,
        device.atcmd(&rename_node.create())?
    );
    println!("{:?}: {:?}", discover, device.atcmd(&discover.create())?);
    device.cmd_mode(false)?;
    Ok(())
}
