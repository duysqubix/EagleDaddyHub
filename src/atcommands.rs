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
    ID(Option<&'a [u8]>),
    AP(Option<&'a [u8]>),
    ND(Option<&'a [u8]>),
    NI(Option<&'a [u8]>),
    CN,
    CmdOn,
}

impl AtCommands<'_> {
    pub fn create(&self) -> AtCommand {
        match *self {
            AtCommands::ID(ref param) => AtCommand {
                command: "ID",
                parameter: param,
                rcr_len: 1,
            },
            AtCommands::CN => AtCommand {
                command: "CN",
                parameter: &None,
                rcr_len: 1,
            },
            AtCommands::CmdOn => AtCommand {
                command: "+++",
                parameter: &None,
                rcr_len: 1,
            },

            AtCommands::AP(ref param) => AtCommand {
                command: "AP",
                parameter: param,
                rcr_len: 1,
            },
            AtCommands::ND(ref param) => AtCommand {
                command: "ND",
                parameter: param,
                rcr_len: 10 + 1,
            },

            AtCommands::NI(ref param) => AtCommand {
                command: "NI",
                parameter: param,
                rcr_len: 1,
            },
        }
    }
}
