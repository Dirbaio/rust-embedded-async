#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

const RX_PIN: usize = 8;
const TX_PIN: usize = 6;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let p = nrf52840_pac::Peripherals::take().unwrap();

    // Configure RX pin
    p.P0.pin_cnf[RX_PIN].write(|w| w.input().connect());
    p.UARTE0.psel.rxd.write(|w| unsafe { w.bits(RX_PIN as _) });

    // Configure TX pin
    p.P0.outset.write(|w| unsafe { w.bits(1 << TX_PIN) });
    p.P0.pin_cnf[TX_PIN].write(|w| w.dir().output());
    p.UARTE0.psel.txd.write(|w| unsafe { w.bits(TX_PIN as _) });

    // Configure baud rate
    p.UARTE0.baudrate.write(|w| w.baudrate().baud115200());

    // Enable
    p.UARTE0.enable.write(|w| w.enable().enabled());

    // Configure buffer for reading
    let mut buf = [0u8; 8];
    p.UARTE0.rxd.ptr.write(|w| unsafe { w.bits(buf.as_mut_ptr() as _) });
    p.UARTE0.rxd.maxcnt.write(|w| unsafe { w.bits(buf.len() as _) });

    // Start read
    info!("Reading...");
    p.UARTE0.tasks_startrx.write(|w| w.tasks_startrx().set_bit());

    // Wait for read to finish.
    while !p.UARTE0.events_endrx.read().events_endrx().bit() {}
    info!("Read done, got {:02x}", buf);

    // Sleep in low-power mode.
    loop {
        cortex_m::asm::wfi();
    }
}
