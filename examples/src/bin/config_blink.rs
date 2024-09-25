//! This example turns on the onboard LED when a target is moving towards the radar.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::UART0,
    uart::{self, BufferedInterruptHandler, BufferedUart},
};

use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use hlk_ld2450::{Desync, NormalMode, RadarError, LD2450};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

use {defmt_rtt as _, panic_probe as _};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // configure buffered UART
    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);
    static TX_BUF: StaticCell<[u8; 48]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 48])[..];
    static RX_BUF: StaticCell<[u8; 48]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 48])[..];

    let mut config = uart::Config::default();
    // The LD2450 uses a default baud rate of 256000, this is a good guess
    // Framing errors are a good indicator of a wrong baud rate
    config.baudrate = 256000;
    config.parity = uart::Parity::ParityNone;
    config.stop_bits = uart::StopBits::STOP1;

    let uart = BufferedUart::new(uart, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, config);

    let mut led = Output::new(p.PIN_25, Level::Low);

    // set LED high while we have bluetooth on
    led.set_high();

    // Use the default configuration, but enable bluetooth
    let radar_config = hlk_ld2450::Config {
        bluetooth_enabled: true,
        ..Default::default()
    };

    let mut radar = LD2450::new(uart, radar_config).await;

    // wait 15 seconds before turning bluetooth off
    Delay.delay_ms(15000).await;

    if Err(RadarError::Desyncronized) == radar.set_bluetooth_enabled(false).await {
        // This is a somewhat serious error, we should probably not continue,
        // or power cycle the radar if possible. However, serial errors are likely
        // an issue with the hardware or connections
        panic!("Unable to recover from error during config mode entry/exit");
    };

    loop {
        // ignore any radar errors
        _ = blink_for_motion(&mut radar, &mut led).await;
    }
}

/// Turns on the LED when a target is moving towards the radar
async fn blink_for_motion<S: embedded_io_async::Read, P: embedded_hal::digital::OutputPin>(
    radar: &mut LD2450<S>,
    led: &mut P,
) -> Result<(), hlk_ld2450::RadarError> {
    let targets = radar.next_radar_targets().await?;
    if targets.iter().any(|t| t.speed < 0) {
        led.set_high().unwrap();
    } else {
        led.set_low().unwrap();
    }

    Ok(())
}
