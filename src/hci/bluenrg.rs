use crate::hci::vendor::Vendor;
use nom::IResult;
use nom::bytes::complete::take;
use nom::combinator::{verify};
use nom::number::complete::le_u16;
use crate::hci::command::Command;

#[derive(Debug)]
pub struct BlueNrg {}

impl BlueNrg {
    fn blue_initialized_event(i: &[u8]) -> IResult<&[u8], <Self as Vendor>::Event> {
        let (i, _code) = verify(le_u16, |code: &u16| *code == 0x0001)(i)?;
        let (i, reason) = Self::blue_initialization_reason(i)?;

        Ok((
            i,
            BlueNrgEvent::BlueInitialized(reason)
        ))
    }

    fn blue_initialization_reason(i: &[u8]) -> IResult<&[u8], BlueInitializedReason> {
        let (i, reason_code) = take(1usize)(i)?;
        let reason = match reason_code[0] {
            0x01 => BlueInitializedReason::FirmwareStartedProperly,
            0x05 => BlueInitializedReason::Reset(ResetCause::Watchdog),
            _ => panic!("not handled")
        };

        Ok((i, reason))
    }
}

impl Vendor for BlueNrg {
    type Opcode = BlueNrgOpcode;
    type Event = BlueNrgEvent;

    fn vendor_event(i: &[u8]) -> IResult<&[u8], Self::Event> {
        let (i, vendor) = verify(take(1usize), |b: &[u8]| b[0] == 0xFF)(i)?;
        log::info!("vendor {:?}", vendor);
        let (i, len) = take(1usize)(i)?;
        log::info!("len {:?}", len);
        Self::blue_initialized_event(i)
    }
}

#[derive(Debug)]
pub enum BlueNrgOpcode {
    GetFirmwareBuildNumber = 0xFC00,
}

pub struct GetFirmwareBuildNumber;

impl Command for GetFirmwareBuildNumber {
    fn opcode(&self) -> u16 {
        BlueNrgOpcode::GetFirmwareBuildNumber as u16
    }

    fn parameters(&self) -> Option<&[u8]> {
        None
    }
}

impl From<BlueNrgOpcode> for u16 {
    fn from(opcode: BlueNrgOpcode) -> Self {
        opcode as u16
    }
}

#[derive(Debug)]
pub enum BlueNrgEvent {
    BlueInitialized(BlueInitializedReason),
}

#[derive(Debug)]
pub enum BlueInitializedReason {
    FirmwareStartedProperly,
    UpdaterModeEntered(UpdaterModeCause),
    Reset(ResetCause),
}

#[derive(Debug)]
pub enum UpdaterModeCause {
    AciUpdaterStart,
    BadBlueFlag,
    IrqPin,
}

#[derive(Debug)]
pub enum ResetCause {
    Watchdog,
    Lockup,
    Brownout,
    Crash,
    EccError,
}

