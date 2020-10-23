use heapless::{
    consts::*,
    spsc::{
        Queue,
        Producer,
        Consumer,
    },
};

use crate::packet::RawPacket;
use crate::hci::opcode::Opcode;
use crate::hci::bluenrg::{BlueNrgOpcode, BlueNrg, GetFirmwareBuildNumber};
use crate::hci::command::Command;
use crate::hci::vendor::Vendor;

pub struct Interface {
    requests: Producer<'static, RawPacket, U16>,
    responses: Consumer<'static, RawPacket, U16>,
}

impl Interface {

    pub fn new(
        requests: Producer<'static, RawPacket, U16>,
        responses: Consumer<'static, RawPacket, U16>,
    ) -> Self {
        Self {
            requests,
            responses,
        }
    }

    pub fn get_firmware_build_version(&mut self) -> Result<(),()> {
        self.send_command( GetFirmwareBuildNumber );
        Ok(())
    }

    pub fn send_command<C: Command>(&mut self, command: C) -> Result<(),()> {
        let packet = command.into();
        self.requests.enqueue(packet);
        Ok(())
    }


}