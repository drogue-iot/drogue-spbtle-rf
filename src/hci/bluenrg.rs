use crate::hci::vendor::Vendor;
use nom::IResult;
use nom::bytes::complete::take;
use nom::combinator::{verify};
use nom::number::complete::le_u16;
use crate::hci::command::Command;
use nom::error::ErrorKind;

#[derive(Debug)]
pub struct BlueNrg {}

impl BlueNrg {
    fn blue_initialized_event(i: &[u8]) -> IResult<&[u8], <Self as Vendor>::Event> {
        log::info!("try reason");
        let (i, _code) = verify(le_u16, |code: &u16| *code == 0x0001)(i)?;
        let (i, reason) = Self::blue_initialization_reason(i)?;

        log::info!("reason {:?}", reason);

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
    type Command = BlueNrgCommand;
    type ReturnParameters = BlueNrgReturnParameters;

    fn vendor_event(i: &[u8]) -> IResult<&[u8], Self::Event> {
        log::info!("try BlueNrg vendor event");
        let (i, vendor) = verify(take(1usize), |b: &[u8]| b[0] == 0xFF)(i)?;
        log::info!("vendor {:?}", vendor);
        let (i, len) = take(1usize)(i)?;
        log::info!("len {:?}", len);
        let r = Self::blue_initialized_event(i);
        log::info!("whut {:?}", r);
        r
    }

    fn return_parameters(opcode: u16, i: &[u8]) -> IResult<&[u8], Self::ReturnParameters> {
        log::info!( "return params {:#x?}", i);
        match opcode {
            0xFC00 => {
                Ok( (&i[3..], BlueNrgReturnParameters::FirmwareBuildNumber {
                    status: i[0],
                    build_number: u16::from_le_bytes([ i[1], i[2] ])
                } ) )
            }
            _ => {
                IResult::Err( nom::Err::Failure( (i, ErrorKind::Tag) ) )
            }
        }
    }
}

#[derive(Debug)]
pub enum BlueNrgCommand {
    GetFirmwareBuildNumber,
}

impl Command for BlueNrgCommand {
    fn opcode(&self) -> u16 {
        match self {
            BlueNrgCommand::GetFirmwareBuildNumber => { 0xFC00 }
        }
    }

    fn parameters(&self) -> Option<&[u8]> {
        match self {
            BlueNrgCommand::GetFirmwareBuildNumber => { None }
        }
    }
}

#[derive(Debug)]
pub enum BlueNrgReturnParameters {
    FirmwareBuildNumber{
        status: u8,
        build_number: u16,
    },
}

#[derive(Debug)]
pub enum BlueNrgOpcode {
    GetFirmwareBuildNumber = 0xFC00,
}

//pub struct GetFirmwareBuildNumber;

#[derive(Debug)]
pub struct FirmwareBuildNumber {
    status: u8,
    build_number: u16,
}

//impl Command for GetFirmwareBuildNumber {
    //type ReturnParameters = FirmwareBuildNumber;
    //fn opcode(&self) -> u16 {
        //BlueNrgOpcode::GetFirmwareBuildNumber as u16
    //}
//
    //fn parameters(&self) -> Option<&[u8]> {
        //None
    //}
//
//}

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

