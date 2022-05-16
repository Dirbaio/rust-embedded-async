#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use embassy::executor::Spawner;
use embassy::time::{with_timeout, Duration};
use embassy_nrf::{interrupt, uarte};

#[embassy::main]
async fn main(_spawner: Spawner, p: embassy_nrf::Peripherals) -> ! {
    info!("Hello World!");

    // Configure UART
    let mut config = uarte::Config::default();
    config.baudrate = uarte::Baudrate::BAUD115200;
    let irq = interrupt::take!(UARTE0_UART0);
    let mut uart = uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_06, config);

    let mut buf = [0u8; 8];

    // Wait for either read done or timeout, whichever comes first.
    match with_timeout(Duration::from_secs(1), uart.read(&mut buf)).await {
        Ok(_) => info!("Read done, got {:02x}", buf),
        Err(_) => info!("Timeout!"),
    }
}
