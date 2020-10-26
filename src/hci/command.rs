use crate::packet::RawPacket;
use crate::hci::opcode::Opcode;
use crate::hci::vendor::Vendor;
use core::fmt::Debug;

pub trait Command {
    type ReturnParameters: Debug;

    fn opcode(&self) -> u16;
    fn parameters(&self) -> Option<&[u8]>;
}

