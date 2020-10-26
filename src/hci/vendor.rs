use nom::IResult;
use core::fmt::Debug;
use crate::hci::command::Command;

pub trait Vendor : Debug {
    type Opcode: Into<u16> + Debug;
    type Event: Debug;
    type ReturnParameters: Debug;
    type Command: Debug + Command;

    fn vendor_event(i: &[u8]) -> IResult<&[u8], Self::Event>;
    fn return_parameters(opcode: u16, i: &[u8] ) -> IResult<&[u8], Self::ReturnParameters>;
}