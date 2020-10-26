use heapless::{
    consts::*,
    Vec,
    spsc::{
        Queue,
        Producer,
        Consumer,
    },
};

use crate::packet::RawPacket;
use crate::hci::opcode::Opcode;
use crate::hci::bluenrg::{BlueNrgOpcode, BlueNrg, BlueNrgCommand};
//use crate::hci::command::Command;
use crate::hci::vendor::Vendor;
use crate::hci::{HciPacket, HciEvent, HciCommand, CoreEvent};

pub struct Interface {
    requests: Producer<'static, HciPacket<BlueNrg>, U16>,
    responses: Consumer<'static, HciPacket<BlueNrg>, U16>,
}

impl Interface {
    pub fn new(
        requests: Producer<'static, HciPacket<BlueNrg>, U16>,
        responses: Consumer<'static, HciPacket<BlueNrg>, U16>,
    ) -> Self {
        Self {
            requests,
            responses,
        }
    }

    pub fn get_firmware_build_version(&mut self) -> Result<(), ()> {
        let result = self.send_command(HciCommand::Vendor(BlueNrgCommand::GetFirmwareBuildNumber));
        Ok(())
    }

    pub fn send_command(&mut self, command: HciCommand<BlueNrg>) -> Result<<BlueNrg as Vendor>::ReturnParameters, ()> {
        let sent_opcode = command.opcode();
        self.requests.enqueue( HciPacket::Command(command));

        loop {
            let packet = self.responses.dequeue();
            if let Some(HciPacket::Event(HciEvent::Core(CoreEvent::CommandComplete { packets, opcode, return_parameters }))) = packet {
                if opcode == sent_opcode {
                    log::info!("RESPONSE ---> {:?} {}", return_parameters, packets);
                    break;
                }
            }
        }
        log::info!("done looping");
        Err(())
    }
}