use nom::IResult;
use nom::bytes::complete::take;
use nom::bytes::complete::tag;
use nom::branch::alt;
use nom::combinator::{verify, map};
use nom::number::complete::{le_u16, le_u8, be_u16};
use nom::error::ErrorKind;

use heapless::Vec;

use crate::hci::{HciPacket, HciEvent, CommandStatusCode, HciReturnParameters, CoreEvent};
use crate::hci::vendor::Vendor;

pub fn parse_packet<V: Vendor>(i: &[u8]) -> Result<HciPacket<V>, ()> {
    let result = packet(i);
    log::info!("parse result {:?}", result);
    match result {
        Ok((_i, packet)) => {
            Ok(packet)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(())
        }
    }
}

pub fn packet<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciPacket<V>> {
    event_packet(i)
}

pub fn event_packet<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciPacket<V>> {
    let (i, _) = verify(take(1usize), |b: &[u8]| b[0] == 0x04)(i)?;
    log::info!("event_packet1");
    let (i, event) = event(i)?;
    log::info!("event_packet2 {:?}", event);
    let r = Ok((i, HciPacket::Event(event)));
    r
}

pub fn event<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciEvent<V>> {
    alt(
        (
            map(V::vendor_event, |event| HciEvent::Vendor(event)),
            map(core_event, |event| HciEvent::Core(event))
        )
    )(i)
}

pub fn core_event<V: Vendor>(i: &[u8]) -> IResult<&[u8], CoreEvent<V>> {
    alt((
        hci_event_command_complete,
        hci_event_command_status,
    ))(i)
}

pub fn hci_event_command_complete<V: Vendor>(i: &[u8]) -> IResult<&[u8], CoreEvent<V>> {
    let (i, _code) = verify(le_u8, |code| *code == 0x0E)(i)?;
    let (i, len) = le_u8(i)?;
    log::info!("event complete len {}", len);
    let (i, packets) = le_u8(i)?;
    let (i, opcode) = le_u16(i)?;

    log::info!(" len {} opcode {:#x}", len, opcode);

    if (opcode & 0xFF00) != 0 {
        let (i, parameters) = V::return_parameters(opcode, i)?;
        Ok((i, CoreEvent::CommandComplete {
            packets,
            opcode,
            return_parameters: HciReturnParameters::Vendor(parameters),
        }))
    } else {
        Ok((i, CoreEvent::CommandComplete {
            packets,
            opcode,
            return_parameters: HciReturnParameters::Hci,
        }))
    }
}

pub fn hci_event_command_status<V: Vendor>(i: &[u8]) -> IResult<&[u8], CoreEvent<V>> {
    let (i, _code) = verify(le_u8, |code| *code == 0x0F)(i)?;
    let (i, len) = le_u8(i)?;
    let (i, packets) = le_u8(i)?;
    let (i, status) = le_u8(i)?;
    let (i, opcode) = le_u16(i)?;

    log::info!("code {:#x} len={}", _code, len);

    Ok((i, CoreEvent::CommandStatus {
        packets,
        status: CommandStatusCode::from(status),
        opcode,
    }))
}


