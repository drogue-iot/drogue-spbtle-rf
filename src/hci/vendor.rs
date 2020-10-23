use nom::IResult;
use core::fmt::Debug;

pub trait Vendor : Debug {
    type Opcode: Into<u16> + Debug;
    type Event: Debug;

    fn vendor_event(i: &[u8]) -> IResult<&[u8], Self::Event>;
}