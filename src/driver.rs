use embedded_hal::blocking::spi::Transfer;
use core::fmt::Debug;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use drogue_embedded_timer::Delay;
use embedded_time::duration::Milliseconds;
use core::marker::PhantomData;
use heapless::{
    consts::*,
    spsc::{
        Queue,
        Consumer,
        Producer,
    },
};
use crate::interface::Interface;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::packet::RawPacket;
use crate::hci::bluenrg::{BlueNrgEvent, BlueNrg};
use crate::hci::HciPacket;
use nom::InputIter;

pub struct Driver<'clock, ChipSelectPin, ResetPin, ReadyPin, Clock>
    where
        ChipSelectPin: OutputPin,
        ChipSelectPin::Error: Debug,
        ResetPin: OutputPin,
        ResetPin::Error: Debug,
        ReadyPin: InputPin,
        ReadyPin::Error: Debug,
        Clock: embedded_time::Clock,
{
    cs: ChipSelectPin,
    reset: ResetPin,
    ready: ReadyPin,
    clock: &'clock Clock,
    //
    requests: Consumer<'static, HciPacket<BlueNrg>, U16>,
    responses: Producer<'static, HciPacket<BlueNrg>, U16>,
    //
    initialized: bool,
}

impl<'clock, ChipSelectPin, ResetPin, ReadyPin, Clock> Driver<'clock, ChipSelectPin, ResetPin, ReadyPin, Clock>
    where
        ChipSelectPin: OutputPin,
        ChipSelectPin::Error: Debug,
        ResetPin: OutputPin,
        ResetPin::Error: Debug,
        ReadyPin: InputPin,
        ReadyPin::Error: Debug,
        Clock: embedded_time::Clock,
{
    pub fn new(cs: ChipSelectPin,
               reset: ResetPin,
               ready: ReadyPin,
               clock: &'clock Clock,
               request_queue: &'static mut Queue<HciPacket<BlueNrg>, U16>,
               response_queue: &'static mut Queue<HciPacket<BlueNrg>, U16>,
    ) -> (Self, Interface) {
        let (request_producer, request_consumer) = request_queue.split();
        let (response_producer, response_consumer) = response_queue.split();
        let mut driver = Self {
            cs,
            reset,
            ready,
            clock,
            //
            requests: request_consumer,
            responses: response_producer,
            //
            initialized: false,
        };

        let mut interface = Interface::new(request_producer, response_consumer);

        (driver, interface)
    }

    fn start(&mut self) -> Result<(), ResetPin::Error> {
        self.reset.set_high()
    }

    fn stop(&mut self) -> Result<(), ResetPin::Error> {
        self.reset.set_low()
    }

    fn reset(&mut self) -> Result<(), ResetPin::Error> {
        self.stop()?;
        Delay::new(self.clock).delay(Milliseconds(5u32));
        self.start()
    }

    fn select(&mut self) -> Result<(), ChipSelectPin::Error> {
        self.cs.set_low()
    }

    fn deselect(&mut self) -> Result<(), ChipSelectPin::Error> {
        self.cs.set_high()
    }

    // ------------------------------------------------------------------------
    // ------------------------------------------------------------------------

    pub fn get_ready_pin(&mut self) -> &mut ReadyPin {
        &mut self.ready
    }

    pub fn process_irq<Spi: Transfer<u8>>(&mut self, spi: &mut Spi) {
        loop {
            let (writeable_len, readable_len) = self.block_until_ready(spi);
            if readable_len > 0 {
                log::info!("irq has readable {}, writable {}", readable_len, writeable_len);
            }

            if readable_len == 0 {
                self.cs.set_high();
                if !self.initialized {
                    continue;
                }
                return
            }
            let mut buf = [0; 257];
            spi.transfer(&mut buf[0..readable_len as usize]);

            log::info!("transfer from {:02X?}", &buf[0..readable_len as usize]);
            let result = crate::hci::parser::parse_packet::<BlueNrg>(&buf[0..readable_len as usize]);
            log::info!("enqueuing ----> {:?}", result);
            if matches!(result, Ok(crate::hci::HciPacket::Event(crate::hci::HciEvent::Vendor(BlueNrgEvent::BlueInitialized(_))))) {
                self.initialized = true
            }
            self.responses.enqueue( result.unwrap() );
            self.cs.set_high();
            return;
        }
    }

    pub fn process_fifo<Spi: Transfer<u8>>(&mut self, spi: &mut Spi)
        where Spi::Error: Debug
    {
        if !self.initialized {
            return;
        }

        while let Some(HciPacket::Command(command)) = self.requests.dequeue() {
            log::info!("fifo: {:?}", command);

            let mut buf = [0; 257];
            let opcode = command.opcode().to_le_bytes();
            let payload = command.parameters();


            buf[0] = 0x01;
            buf[1] = opcode[0];
            buf[2] = opcode[1];

            let mut len = 4;

            if let Some(payload) = payload {
                buf[3] = payload.len() as u8;
                for (i, b) in payload.iter().enumerate() {
                    buf[len] = *b;
                    len += 1
                }
            } else {
                buf[3] = 0;
            }

            log::info!("command to send >> {:#x?} {}", &buf[..len], len);
            let (writable_len, readable_len) = self.block_until_writable(spi, len);
            log::info!(" writable {}, readable {}", writable_len, readable_len);
            log::info!(" sending {:#x?}", payload);

            let result = spi.transfer(&mut buf[..len]);
            log::info!("result -> {:?}", result);

            self.cs.set_high();
        }

        self.process_irq(spi)
    }

    fn block_until_writable<Spi: Transfer<u8>>(&mut self, spi: &mut Spi, num: usize) -> (u16, u16) {
        self.cs.set_low();

        loop {
            let mut buf = [WRITE, 0, 0, 0, 0];

            spi.transfer(&mut buf);

            if buf[0] == 0x02 {
                let writeable_len = u16::from_le_bytes([buf[1], buf[2]]);
                let readable_len = u16::from_le_bytes([buf[3], buf[4]]);
                if writeable_len as usize >= num {
                    return (writeable_len, readable_len);
                }
            }
            self.cs.set_high();
            Delay::new(self.clock).delay(Milliseconds(1u32));
            self.cs.set_low();
        }
    }

    fn block_until_ready<Spi: Transfer<u8>>(&mut self, spi: &mut Spi) -> (u16, u16) {
        self.cs.set_low();

        loop {
            let mut buf = [READ, 0, 0, 0, 0];

            spi.transfer(&mut buf);

            if buf[0] == 0x02 {
                let writeable_len = u16::from_le_bytes([buf[1], buf[2]]);
                let readable_len = u16::from_le_bytes([buf[3], buf[4]]);
                return (writeable_len, readable_len);
            }
            self.cs.set_high();
            Delay::new(self.clock).delay(Milliseconds(1u32));
            self.cs.set_low();
        }
    }
}


const WRITE: u8 = 0x0A;
const READ: u8 = 0x0B;

