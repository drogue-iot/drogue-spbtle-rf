use crate::hci::vendor::Vendor;
use heapless::{
    consts::*,
    Vec
};

pub mod vendor;
pub(crate) mod parser;
pub(crate) mod opcode;
pub(crate) mod command;
pub mod bluenrg;

#[derive(Debug)]
pub enum Packet<V: Vendor> {
    Event(Event<V>),
}

#[derive(Debug)]
pub enum Event<V: Vendor> {
    Hci(HciEvent<V>),
    Vendor(V::Event),
}

#[derive(Debug)]
pub enum HciEvent<V: Vendor> {
    CommandComplete { packets: u8, opcode: u16, return_parameters: ReturnParameters<V> },
    CommandStatus { status: CommandStatusCode, packets: u8, opcode: u16 },
}

#[derive(Debug)]
pub enum ReturnParameters<V: Vendor> {
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
