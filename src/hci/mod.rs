use crate::hci::vendor::Vendor;
use heapless::{
    consts::*,
    Vec
};
use crate::hci::command::Command;

pub mod vendor;
pub(crate) mod parser;
pub(crate) mod opcode;
pub(crate) mod command;
pub mod bluenrg;

#[derive(Debug)]
pub enum HciPacket<V: Vendor> {
    Command(HciCommand<V>),
    Event(HciEvent<V>),
}

#[derive(Debug)]
pub enum HciCommand<V:Vendor> {
    Hci,
    Vendor(V::Command),
}

impl<V:Vendor> HciCommand<V> {
    pub fn opcode(&self) -> u16 {
        match self {
            HciCommand::Hci => {
                unimplemented!()
            }
            HciCommand::Vendor(vc) => {
                vc.opcode()
            }
        }
    }

    pub fn parameters(&self) -> Option<&[u8]> {
        match self {
            HciCommand::Hci => {
                unimplemented!()
            }
            HciCommand::Vendor(vc) => {
                vc.parameters()
            }
        }
    }
}

#[derive(Debug)]
pub enum HciEvent<V: Vendor> {
    Core(CoreEvent<V>),
    Vendor(V::Event),
}

#[derive(Debug)]
pub enum CoreEvent<V: Vendor> {
    CommandComplete { packets: u8, opcode: u16, return_parameters: HciReturnParameters<V> },
    CommandStatus { status: CommandStatusCode, packets: u8, opcode: u16 },
}

#[derive(Debug)]
pub enum HciReturnParameters<V: Vendor> {
    Hci,
    Vendor(V::ReturnParameters),
}

#[derive(Debug)]
pub enum CommandStatusCode {
    UnknownHciCommand = 0x01,
    UnknownConnectionIdentifier = 0x02,
    HardwareFailure = 0x03,
    PageTimeout = 0x04,
    AuthenticationFailure = 0x05,
}

impl From<u8> for CommandStatusCode {
    fn from(code: u8) -> Self {
        match code {
            0x01 => CommandStatusCode::UnknownHciCommand,
            _ => panic!("unhandled status")
        }
    }
}
