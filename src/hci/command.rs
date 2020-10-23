use crate::packet::RawPacket;
use crate::hci::opcode::Opcode;
use crate::hci::vendor::Vendor;

pub trait Command {
    //fn opcode(&self) -> V::Opcode;
    fn opcode(&self) -> u16;
    fn parameters(&self) -> Option<&[u8]>;
}

