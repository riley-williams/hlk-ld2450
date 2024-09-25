//! This example logs radar targets to the USB serial port.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::{UART0, USB},
    uart::{self, BufferedInterruptHandler, BufferedUartRx},
    usb::{Driver, InterruptHandler},
};
use hlk_ld2450::LD2450;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(4096, log::LevelFilter::Info, driver);
}

use {defmt_rtt as _, panic_probe as _};
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // set up USB serial logging
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Configure buffered UART with only Rx.
    // Only Rx is needed here because we reuse whatever config was already set
    // on the device. Normally this is only done for demonstration.

    let (rx_pin, uart) = (p.PIN_1, p.UART0);
    static RX_BUF: StaticCell<[u8; 48]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 48])[..];

    let mut config = uart::Config::default();
    // The LD2450 uses a default baud rate of 256000, this is a good guess
    // Framing errors are a good indicator of a wrong baud rate
    config.baudrate = 256000;
    config.parity = uart::Parity::ParityNone;
    config.stop_bits = uart::StopBits::STOP1;

    let uart = BufferedUartRx::new(uart, Irqs, rx_pin, rx_buf, config);
    let mut radar = LD2450::new_recycled_config(uart);

    loop {
        match radar.next_radar_targets().await {
            Ok(targets) => {
                log::info!("{} targets: {:?}", targets.len(), targets);
            }
            Err(e) => log::error!("Error: {:?}", e),
        }
    }
}
