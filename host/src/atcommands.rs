//!
//! List of valid ATCommands
//!
//!

pub struct AtCommand<'a> {
    pub command: &'a str,
    pub parameter: &'a Option<&'a [u8]>,
    pub rcr_len: usize, // the number of carriage returns in the reponse for this command
}

#[derive(Debug)]
pub enum AtCommands<'a> {
    //    AC,                     // apply changes
    //    AP(Option<&'a [u8]>),   // API
    //    CmdOn,
    //    CN,
    //    ID(Option<&'a [u8]>),
    Discover(Option<&'a [u8]>),
    //    NI(Option<&'a [u8]>),
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
