//!
//!
//!  Interactive Console mode.
//!
//!  Scan modules and interact with connected modules
//!
//!

use crate::manager::{self, ModuleManager};
use chrono::{NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;
use rustbee::device;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{self, Write};

pub type Result<T> = std::result::Result<T, Error>;
pub type ProcessCmd = fn(&mut Console, &Args) -> Result<()>;

lazy_static! {
    pub static ref COMMAND_MAP: HashMap<&'static str, (ProcessCmd, &'static str)> = {
        let mut map: HashMap<&'static str, (ProcessCmd, &'static str)> = HashMap::new();
        map.insert("exit", (do_exit, "Exit interactive mode"));
        map.insert("help", (do_help, "Display this screen"));
        map.insert("list", (do_list, "List all connected modules"));
        map.insert("clear", (do_clear, "Clear the screen"));
        map.insert(
            "discover",
            (do_discovery, "Discover Modules on the network"),
        );
        map.insert(
            "load",
            (
                do_load_modules,
                "If saved, load previously saved modules into memory",
            ),
        );

        map.insert("save", (do_save_modules, "Save current modules to disk"));
        map.insert("send", (do_module_send, "Send explicit commands to module"));
        map
    };
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ManagerError(manager::Error),
    DeviceError(device::Error),
    InvalidCommand,
    InvalidSubArgs(String),
    EmptyInput,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::IOError(ref err) => write!(f, "{}", err),
            Error::InvalidCommand => write!(f, "Invalid Command"),
            Error::EmptyInput => write!(f, ""),
            Error::ManagerError(ref err) => write!(f, "{}", err),
            Error::DeviceError(ref err) => write!(f, "{}", err),

            Error::InvalidSubArgs(ref err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<manager::Error> for Error {
    fn from(err: manager::Error) -> Error {
        Error::ManagerError(err)
    }
}

impl From<device::Error> for Error {
    fn from(err: device::Error) -> Error {
        Error::DeviceError(err)
    }
}

fn do_module_send(con: &mut Console, args: &Args) -> Result<()> {
    if args.sub_args.len() < 3 {
        return Err(Error::InvalidSubArgs(
            "Invalid subarguments length".to_string(),
        ));
    }

    let node_ids = con
        .manager
        .get_node_ids()?
        .iter()
        .map(|s| s.to_lowercase())
        .collect::<Vec<String>>();
    let device_id = &args.sub_args[0];
    let device_action = &args.sub_args[1];

    if node_ids.contains(&device_id) == true {
        // appropriate slave device

        if &device_action[..] == "request" {
            let request_option = &args.sub_args[2];
            let selected_module = con.manager.get_module(device_id);

            if let None = selected_module {
                return Err(Error::ManagerError(manager::Error::NoDetectedModules));
            }
            let module_idx = selected_module.unwrap();
            match &request_option[..] {
                "th" => {
                    let response = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestTH)?;

                    let temp =
                        f32::from_le_bytes(<[u8; 4]>::try_from(&response.rf_data[0..4]).unwrap());
                    let humidity =
                        f32::from_le_bytes(<[u8; 4]>::try_from(&response.rf_data[4..8]).unwrap());

                    println!("Temperature: {}C\nHumdity: {}%", temp, humidity);
                }
                "time" => {
                    let response = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestTime)?;
                    // firs two bytes is module_id
                    let ref d = response.rf_data;
                    let sec = d[2];
                    let min = d[3];
                    let hour = d[4];
                    let day = d[5];
                    let month = d[6];
                    let year = ((d[7] as u16) << 8) | (d[8] as u16);

                    let module_time: NaiveDateTime = NaiveDate::from_ymd(
                        year as i32,
                        month as u32,
                        day as u32,
                    )
                    .and_hms(hour as u32, min as u32, sec as u32);
                    println!("Module Time: {:?}", module_time);
                }
                "dist" => {
                    let response = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestDist)?;

                    let dist_cm: f32 =
                        f32::from_le_bytes(<[u8; 4]>::try_from(&response.rf_data[2..6]).unwrap());
                    println!(
                        "Detected Distance: {:.2}cm/{:.2}in",
                        dist_cm,
                        ((dist_cm as f32) * 0.39)
                    );
                }
                "motor" => {
                    let _response = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestMotor)?;
                }

                "all" => {
                    let th = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestTH)?;

                    let temp = f32::from_le_bytes(<[u8; 4]>::try_from(&th.rf_data[0..4]).unwrap());
                    let humidity =
                        f32::from_le_bytes(<[u8; 4]>::try_from(&th.rf_data[4..8]).unwrap());

                    println!("Temperature: {}C\nHumdity: {}%", temp, humidity);

                    let time = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestTime)?;
                    // firs two bytes is module_id
                    let ref d = time.rf_data;
                    let sec = d[2];
                    let min = d[3];
                    let hour = d[4];
                    let day = d[5];
                    let month = d[6];
                    let year = ((d[7] as u16) << 8) | (d[8] as u16);

                    let module_time: NaiveDateTime = NaiveDate::from_ymd(
                        year as i32,
                        month as u32,
                        day as u32,
                    )
                    .and_hms(hour as u32, min as u32, sec as u32);
                    println!("Module Time: {:?}", module_time);

                    let dist = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::RequestDist)?;

                    let dist_cm: f32 =
                        f32::from_le_bytes(<[u8; 4]>::try_from(&dist.rf_data[2..6]).unwrap());
                    println!(
                        "Detected Distance: {:.2}cm/{:.2}in",
                        dist_cm,
                        ((dist_cm as f32) * 0.39)
                    );
                }

                "invalid" => {
                    let _response = con
                        .manager
                        .request(module_idx, manager::ModuleCommands::InvalidCmd)?;
                }

                _ => (),
            }
        } else if &device_action[..] == "set" {
            let set_option = &args.sub_args[2];
            let selected_module = con.manager.get_module(device_id);
            if let None = selected_module {
                return Err(Error::ManagerError(manager::Error::NoDetectedModules));
            }
            let module_idx = selected_module.unwrap();

            match &set_option[..] {
                "time" => {
                    let response = con
                        .manager
                        .set(module_idx, manager::ModuleCommands::SetTime)?;
                    println!("{:?}", response.rf_data);
                }
                _ => (),
            }
        } else {
            return Err(Error::InvalidSubArgs(
                "not a valid action for node_id".to_string(),
            ));
        }
    }

    Ok(())
}

/// save current modules to disk
fn do_save_modules(con: &mut Console, _args: &Args) -> Result<()> {
    if con.manager.modules.len() == 0 {
        println!("no modules found");
    } else {
        con.manager.dump_to_disk()?;
        println!("saved");
    }
    Ok(())
}
/// Load modules back into memory
fn do_load_modules(con: &mut Console, _args: &Args) -> Result<()> {
    con.manager.load_modules()?;
    println!("loaded");

    Ok(())
}

/// discover new nodes on network
fn do_discovery(con: &mut Console, args: &Args) -> Result<()> {
    con.manager.discovery_mode()?;

    if args.sub_args.len() > 0 {
        let subcmds: Vec<&str> = args.sub_args.iter().map(|s| s as &str).collect();

        if subcmds.contains(&"save") {
            // after discovery save devices to disk
            do_save_modules(con, args)?;
        }
    }

    Ok(())
}

///clear screen
fn do_clear(_con: &mut Console, _args: &Args) -> Result<()> {
    println!("\x1B[2J");
    Ok(())
}

/// exit interactive mode
fn do_exit(con: &mut Console, _args: &Args) -> Result<()> {
    con.repl_loop = false;
    println!("goodbye");
    Ok(())
}

/// print all valid commands
fn do_help(_con: &mut Console, _args: &Args) -> Result<()> {
    let mut help_str = String::with_capacity(1024);
    help_str.push_str("Module Manager Console v1.0\n\n");
    help_str.push_str("Valid Commands: \n");
    let mut all_cmds: Vec<_> = COMMAND_MAP.keys().collect();
    all_cmds.sort();
    for cmd in all_cmds.iter() {
        let (_, desc) = COMMAND_MAP.get(*cmd).unwrap();
        help_str.push_str(&format!("{} - {}\n", cmd, desc));
    }
    println!("{}", help_str);
    Ok(())
}

/// show list of all connected modules
fn do_list(con: &mut Console, args: &Args) -> Result<()> {
    if args.sub_args.len() > 0 {
        let subcmds: Vec<&str> = args.sub_args.iter().map(|s| s as &str).collect();

        if subcmds.contains(&"clear") {
            // clear current list and return
            con.manager.modules.clear();
            println!("cleared");
            return Ok(());
        }
    }

    let ref module_list = con.manager.modules;

    if module_list.len() == 0 {
        return Err(Error::ManagerError(manager::Error::NoDetectedModules));
    }
    let mut list = String::with_capacity(1024);
    for module in con.manager.modules.iter() {
        list.push_str(&format!("{:#?}\n", module));
    }
    println!("\nModules:\n{}", list);
    Ok(())
}

#[derive(Debug)]
pub struct Args {
    pub main_arg: String,
    pub sub_args: Vec<String>,
}

impl Args {
    fn new(cmds: Vec<&str>) -> Self {
        let mut subcmds: Vec<String> = Vec::new();
        if cmds.len() == 0 {
            return Self {
                main_arg: "".to_string(),
                sub_args: Vec::new(),
            };
        }

        for subcmd in cmds.iter() {
            // if &subcmd[0..1] == "-" {
            //     subcmds.push(subcmd.to_string());
            // }
            subcmds.push(subcmd.to_string());
        }
        subcmds.remove(0);
        Self {
            main_arg: String::from(cmds[0]),
            sub_args: subcmds,
        }
    }
}

#[derive(Debug)]
pub struct Console {
    pub prompt: String,
    pub input_buf: String,
    pub manager: ModuleManager,
    repl_loop: bool,
}

impl Console {
    pub fn new() -> Result<Self> {
        Ok(Self {
            prompt: "mm> ".to_string(),
            input_buf: String::new(),
            manager: ModuleManager::new("COM1", 115200)?,
            repl_loop: true,
        })
    }

    pub fn repl(&mut self) -> Result<()> {
        loop {
            if self.repl_loop == false {
                break;
            }
            self.input_buf.clear();
            print!("{}", self.prompt);
            io::stdout().flush()?;
            io::stdin().read_line(&mut self.input_buf)?;
            if let Err(err) = self.process_input() {
                println!("{}", err);
            }
        }
        Ok(())
    }

    fn process_input(&mut self) -> Result<()> {
        let cmds = self.input_buf.to_lowercase();
        let cmds = cmds.split_whitespace();
        let cmds = cmds.collect::<Vec<&str>>();
        if cmds.len() > 0 {
            let result = match COMMAND_MAP.get(cmds[0]) {
                Some(func) => {
                    let args = Args::new(cmds);
                    func.0(self, &args)
                }
                None => Err(Error::InvalidCommand),
            };
            return result;
        }
        Err(Error::EmptyInput)
    }
}
