use nom::IResult;
use nom::bytes::complete::take;
use nom::bytes::complete::tag;
use nom::branch::alt;
use nom::combinator::{verify, map};
use nom::number::complete::{le_u16, le_u8, be_u16};
use nom::error::ErrorKind;

use heapless::Vec;

use crate::hci::{Packet, Event, HciEvent, CommandStatusCode, ReturnParameters};
use crate::hci::vendor::Vendor;

pub fn parse_packet<V: Vendor>(i: &[u8]) -> Result<Packet<V>, ()> {
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

pub fn packet<V: Vendor>(i: &[u8]) -> IResult<&[u8], Packet<V>> {
    event_packet(i)
}

pub fn event_packet<V: Vendor>(i: &[u8]) -> IResult<&[u8], Packet<V>> {
    let (i, _) = verify(take(1usize), |b: &[u8]| b[0] == 0x04)(i)?;
    log::info!("event_packet1");
    let (i, event) = event(i)?;
    log::info!("event_packet2 {:?}", event);
    let r = Ok((i, Packet::Event(event)));
    r
}

pub fn event<V: Vendor>(i: &[u8]) -> IResult<&[u8], Event<V>> {
    alt(
        (
            map(V::vendor_event, |event| Event::Vendor(event)),
            map(hci_event, |event| Event::Hci(event))
        )
    )(i)
}

pub fn hci_event<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciEvent<V>> {
    alt((
        hci_event_command_complete,
        hci_event_command_status,
    ))(i)
}

pub fn hci_event_command_complete<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciEvent<V>> {
    let (i, _code) = verify(le_u8, |code| *code == 0x0E)(i)?;
    let (i, len) = le_u8(i)?;
    log::info!("event complete len {}", len);
    let (i, packets) = le_u8(i)?;
    let (i, opcode) = le_u16(i)?;

    log::info!(" len {} opcode {:#x}", len, opcode);

    if (opcode & 0xFF00) != 0 {
        let (i, parameters) = V::return_parameters(opcode, i)?;
        Ok((i, HciEvent::CommandComplete {
            packets,
            opcode,
            return_parameters: ReturnParameters::Vendor(parameters),
        }))

    } else {
        Ok((i, HciEvent::CommandComplete {
            packets,
            opcode,
            return_parameters: ReturnParameters::Hci,
        }))

    }

}

pub fn hci_event_command_status<V: Vendor>(i: &[u8]) -> IResult<&[u8], HciEvent<V>> {
    let (i, _code) = verify(le_u8, |code| *code == 0x0F)(i)?;
    let (i, len) = le_u8(i)?;
    let (i, packets) = le_u8(i)?;
    let (i, status) = le_u8(i)?;
    let (i, opcode) = le_u16(i)?;

    log::info!("code {:#x} len={}", _code, len);

    Ok((i, HciEvent::CommandStatus {
        packets,
        status: CommandStatusCode::from(status),
        opcode,
    }))
}


