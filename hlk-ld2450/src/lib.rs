#![no_std]

pub mod config;
mod config_writer;
mod firmware_version;
mod radar_target;

use core::{fmt, marker::PhantomData};

pub use config::Config;
pub use firmware_version::FirmwareVersion;
pub use radar_target::RadarTarget;

use embedded_io_async::{Read, Write};
use radar_target::decode_radar_targets;

const RADAR_DATA_HEADER: [u8; 4] = [0xAA, 0xFF, 0x03, 0x00];
const RADAR_DATA_EOF: [u8; 2] = [0x55, 0xCC];
const RADAR_DATA_FRAME_SIZE: usize = 24;

const RADAR_ACK_HEADER: [u8; 4] = [0xFD, 0xFC, 0xFB, 0xFA];
const RADAR_ACK_EOF: [u8; 4] = [0x04, 0x03, 0x02, 0x01];

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Default)]
pub enum BaudRate {
    Baud9600,
    Baud19200,
    Baud38400,
    Baud57600,
    Baud115200,
    Baud230400,
    #[default]
    Baud256000,
    Baud460800,
}

impl BaudRate {
    fn byte_repr(&self) -> u16 {
        match self {
            BaudRate::Baud9600 => 0x01,
            BaudRate::Baud19200 => 0x02,
            BaudRate::Baud38400 => 0x03,
            BaudRate::Baud57600 => 0x04,
            BaudRate::Baud115200 => 0x05,
            BaudRate::Baud230400 => 0x06,
            BaudRate::Baud256000 => 0x07,
            BaudRate::Baud460800 => 0x08,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadarError {
    /// This is thrown when the driver reaches the end of
    /// a frame without the corresponding EOF token
    UnexpectedFrameSize,
    SerialError,
    /// The radar may ave been left in the config state
    /// due to a serial error during a state change.
    /// This might resolve itself???  TODO: idk
    Desyncronized,
}

pub struct NormalMode;
pub struct ConfigurationMode;
/// The serial port threw errors during a state change and it is
/// unclear whether the radar is in normal or configuration mode
pub struct Desync;

impl core::fmt::Debug for Desync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the device encountered a serial error during mode switch")
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Driver for the LD2450 radar module
pub struct LD2450<Serial> {
    serial: Serial,
}

impl<Serial: Read> LD2450<Serial> {
    /// Initializes the radar. Whatever configuration was previously
    /// set will be reused.
    ///
    /// This method of initialization is not recommended because settings
    /// are persisted even across power cycles of the radar, which can lead
    /// to unexpected behavior. However, it can be useful for testing and
    /// is available without providing a Write pin.
    pub fn new_recycled_config(serial: Serial) -> Self {
        Self { serial }
    }

    /// Reads the radar tracking data
    pub async fn next_radar_targets(
        &mut self,
    ) -> Result<heapless::Vec<RadarTarget, 3>, RadarError> {
        let mut buf = [0; RADAR_DATA_FRAME_SIZE];

        // seek to the start of the next frame
        let mut i = 0;
        while i < RADAR_DATA_HEADER.len() {
            self.serial.read(&mut buf[i..i + 1]).await.map_err(|e| {
                log::error!("{:?}", e);
                RadarError::SerialError
            })?;

            if RADAR_DATA_HEADER[i] != buf[i] {
                // reset the search, potentially catching the new start
                if RADAR_DATA_HEADER[0] == buf[i] {
                    i = 1;
                } else {
                    i = 0;
                }
                continue;
            }
            i += 1;
        }

        // read the rest of the frame, overwriting the header
        self.serial.read_exact(&mut buf).await.map_err(|e| {
            log::error!("{:?}", e);
            RadarError::SerialError
        })?;

        // Read the last two EOF bytes as essentially a sanity check
        let mut throwaway = [0; 2];
        self.serial
            .read_exact(&mut throwaway[..])
            .await
            .map_err(|e| {
                log::error!("{:?}", e);
                RadarError::SerialError
            })?;
        if throwaway != RADAR_DATA_EOF {
            return Err(RadarError::UnexpectedFrameSize);
        }

        decode_radar_targets(&buf)
    }
}

impl<Serial: Read + Write> LD2450<Serial> {
    /// Initialize the radar with a given serial port and configuration.
    /// This is the preferred method of initialization.
    pub async fn new(serial: Serial, config: Config) -> Self {
        // TODO: Set configuration
        Self { serial }
    }

    pub async fn reboot(&mut self) -> Result<(), RadarError> {
        todo!()
    }

    /// Perform a factory reset on the radar. This will reset all settings to
    /// their default, and reboot the radar. Beware applying changes in serial baud rate.
    pub async fn factory_reset(mut self) -> (Serial, Result<(), RadarError>) {
        todo!();

        (self.serial, Ok(()))
    }

    pub async fn firmware_version(&mut self) -> Result<&str, RadarError> {
        todo!()
    }

    pub async fn set_bluetooth_enabled(&mut self, enabled: bool) -> Result<(), RadarError> {
        config_writer::enter_config_mode(&mut self.serial)
            .await
            .map_err(|_| RadarError::Desyncronized)?;
        let result = config_writer::set_bluetooth_enabled(&mut self.serial, enabled).await;
        config_writer::enter_config_mode(&mut self.serial)
            .await
            .map_err(|_| RadarError::Desyncronized)?;

        config_writer::ack(&mut self.serial, &mut [0; 2])
            .await
            .map_err(|_| RadarError::Desyncronized)?;
        result.map_err(|_| RadarError::SerialError)
    }

    pub async fn set_serial_baud_rate(&mut self, baud_rate: BaudRate) -> Result<(), RadarError> {
        let _config_byte = baud_rate.byte_repr();
        todo!()
    }

    pub async fn set_zone_filtering(
        &mut self,
        _zone: u8,
        _enabled: bool,
    ) -> Result<(), RadarError> {
        todo!()
    }
}

impl<Serial> LD2450<Serial> {
    /// Consumes the driver and returns the inner serial port
    pub fn into_inner(self) -> Serial {
        self.serial
    }
}
