#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy::util::{select, Either};
use embassy::{executor::Spawner, waitqueue::AtomicWaker};
use nrf52840_pac::interrupt;

const RX_PIN: usize = 8;
const TX_PIN: usize = 6;

#[embassy::main]
async fn main(_spawner: Spawner, _p: embassy_nrf::Peripherals) -> ! {
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

    // Enable interrupt
    p.UARTE0.intenset.write(|w| w.endrx().set_bit());
    unsafe { cortex_m::peripheral::NVIC::unmask(nrf52840_pac::Interrupt::UARTE0_UART0) };

    // Start read
    info!("Reading...");
    p.UARTE0.tasks_startrx.write(|w| w.tasks_startrx().set_bit());

    // Start timer for 1s.
    p.TIMER0.mode.write(|w| w.mode().timer());
    p.TIMER0.bitmode.write(|w| w.bitmode()._32bit());
    p.TIMER0.cc[0].write(|w| unsafe { w.bits(1_000_000) }); // 1 second
    p.TIMER0.tasks_start.write(|w| w.tasks_start().set_bit());
    p.TIMER0.intenset.write(|w| w.compare0().set_bit());
    unsafe { cortex_m::peripheral::NVIC::unmask(nrf52840_pac::Interrupt::TIMER0) };

    // Wait for either read done or timeout, whichever comes first.
    match select(UartFuture, TimerFuture).await {
        Either::First(_) => info!("Read done, got {:02x}", buf),
        Either::Second(_) => info!("Timeout!"),
    }
}

// ==============================

static UART_WAKER: AtomicWaker = AtomicWaker::new();

struct UartFuture;
impl Future for UartFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        UART_WAKER.register(cx.waker());

        let p = unsafe { nrf52840_pac::Peripherals::steal() };
        if p.UARTE0.events_endrx.read().events_endrx().bit() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for UartFuture {
    fn drop(&mut self) {
        let p = unsafe { nrf52840_pac::Peripherals::steal() };
        p.UARTE0.tasks_stoprx.write(|w| w.tasks_stoprx().set_bit());
    }
}

#[interrupt]
fn UARTE0_UART0() {
    let p = unsafe { nrf52840_pac::Peripherals::steal() };

    if p.UARTE0.events_endrx.read().events_endrx().bit() {
        p.UARTE0.intenclr.write(|w| w.endrx().set_bit());
        UART_WAKER.wake();
    }
}

// ==============================

static TIMER_WAKER: AtomicWaker = AtomicWaker::new();

struct TimerFuture;
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        TIMER_WAKER.register(cx.waker());

        let p = unsafe { nrf52840_pac::Peripherals::steal() };
        if p.TIMER0.events_compare[0].read().events_compare().bit() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for TimerFuture {
    fn drop(&mut self) {
        let p = unsafe { nrf52840_pac::Peripherals::steal() };
        p.TIMER0.tasks_stop.write(|w| w.tasks_stop().set_bit());
    }
}

#[interrupt]
fn TIMER0() {
    let p = unsafe { nrf52840_pac::Peripherals::steal() };

    if p.TIMER0.events_compare[0].read().events_compare().bit() {
        p.TIMER0.intenclr.write(|w| w.compare0().set_bit());
        TIMER_WAKER.wake();
    }
}
