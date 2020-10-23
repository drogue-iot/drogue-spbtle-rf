use crate::hci::vendor::Vendor;

pub mod vendor;
pub(crate) mod parser;
pub(crate) mod opcode;
pub(crate) mod command;
pub(crate) mod bluenrg;

#[derive(Debug)]
pub enum Packet<V: Vendor> {
    Event(Event<V>),
}

#[derive(Debug)]
pub enum Event<V: Vendor> {
    Hci(HciEvent),
    Vendor(V::Event),
}

#[derive(Debug)]
pub enum HciEvent {
    CommandComplete { packets: u8, opcode: u16 },
    CommandStatus { status: CommandStatusCode, packets: u8, opcode: u16 },
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
