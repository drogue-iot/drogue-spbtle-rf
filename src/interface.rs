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
use crate::hci::bluenrg::{BlueNrgOpcode, BlueNrg, GetFirmwareBuildNumber};
use crate::hci::command::Command;
use crate::hci::vendor::Vendor;
use crate::hci::{
    Packet,
    Event,
    HciEvent,
};

pub struct Interface {
    requests: Producer<'static, RawPacket, U16>,
    responses: Consumer<'static, Packet<BlueNrg>, U16>,
}

impl Interface {
    pub fn new(
        requests: Producer<'static, RawPacket, U16>,
        responses: Consumer<'static, Packet<BlueNrg>, U16>,
    ) -> Self {
        Self {
            requests,
            responses,
        }
    }

    pub fn get_firmware_build_version(&mut self) -> Result<(), ()> {
        self.send_command(GetFirmwareBuildNumber);
        Ok(())
    }

    pub fn send_command<C: Command>(&mut self, command: C) -> Result<C::ReturnParameters, ()> {
        let sent_opcode = command.opcode();
        let packet = command.into();
        self.requests.enqueue(packet);

        loop {
            let packet = self.responses.dequeue();
            if let Some(Packet::Event(Event::Hci(HciEvent::CommandComplete{ packets, opcode, return_parameters } ))) = packet {
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