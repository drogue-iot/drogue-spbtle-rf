use heapless::{
    Vec,
    consts::*,
};
use crate::hci::vendor::Vendor;
use crate::hci::command::Command;

type Payload = Vec<u8, U256>;

//#[derive(Debug)]
//pub enum PacketType {
    //Command = 0x01,
    //Event = 0x04,
//}

#[derive(Debug)]
//pub struct RawPacket {
    //packet_type: PacketType,
    //payload: Payload,
//}

pub enum RawPacket {
    Command(Payload),
    Event(Payload),
}

//impl RawPacket {
    //pub fn command(payload: Payload) -> Self {
        //Self {
            //packet_type: PacketType::Command,
            //payload,
        //}
    //}
//}

impl<C: Command> From<C> for RawPacket
{
    fn from(command: C) -> Self {
        let mut payload: Payload = Vec::new();
        let opcode = &command.opcode().to_le_bytes();
        for b in opcode {
            payload.push(*b);
        }

        if let Some(parameters) = command.parameters() {
            payload.extend_from_slice(parameters);
        } else {
            payload.push(0);
        }

        RawPacket::Command(payload)
        //Self {
            //packet_type: PacketType::Command,
            //payload,
        //}
    }
}

