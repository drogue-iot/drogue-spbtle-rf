
use crate::hci::vendor::Vendor;

#[derive(Debug)]
pub enum Opcode<V: Vendor> {
    Spec(SpecOpcode),
    Vendor(V::Opcode),
}

#[derive(Debug)]
pub enum SpecOpcode {

}

/*
impl Opcode {
    pub fn get_bytes(&self, buf: &mut [u8]) -> usize {
        match self {
            Opcode::GetFirmwareBuildNumber => {
                buf[0] = 0x01;
                buf[1] = 0x00;
                buf[2] = 0xFC;
                buf[3] = 0x00; // no param
                4
            }
        }
    }
}

impl From<u16> for Opcode {
    fn from(opcode: u16) -> Self {
        match opcode {
            0xFC00 => Opcode::GetFirmwareBuildNumber,
            _ => unimplemented!()
        }
    }
}
 */