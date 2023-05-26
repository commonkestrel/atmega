//! Driver for the nRF24L01.
//! 
//! Adapted from the [`RF24`](https://www.arduino.cc/reference/en/libraries/rf24/) Arduino library.

use crate::libraries::spi::{ self, SPISettings, BitOrder, DataMode };
use crate::buffer::Buffer;
use crate::buf;
use crate::wiring::Pin;

const W_REGISTER: u8 = 0x20;
const R_REGISTER: u8 = 0x00;
const RF24_SPI_SPEED: u32 = 4_000_000;

static SPI_SETTING_DEFAULT: SPISettings = SPISettings::new(RF24_SPI_SPEED, BitOrder::LSBFirst, DataMode::Mode0);

/// Describes the output power amplification of the antenna.
/// Lower powers have shorter range, but consume less power.
/// 
/// Only affects the nRF24L01 in `TX` mode.
/// 
/// ### PA comparison
/// 
/// | [`PowerAmp`] | `RF_PWR` | RF output power | DC current consumption |
/// | :-- | :-- | :-- | :-- |
/// | [`PowerAmp::Min`] | `00` | -18 dBm | 7.0 mA |
/// | [`PowerAmp::Low`] | `01` | -12 dBm | 7.5 mA |
/// | [`PowerAmp::High`] | `10` | -6 dBm | 9.0 mA |
/// | [`PowerAmp::Max`] | `11` | 0 dBm | 11.3 mA |
pub enum PowerAmp {
    /// -18 dBm output with 7.0 mA current draw.
    Min,
    /// -12 dBm output with 7.5 mA current draw.
    Low,
    /// -6 dBm output with 9.0 mA current draw.
    High,
    /// 0 dBm output with 11.3 mA current draw.
    Max,
}

/// How fast data moves through the air.
/// Units are in bits per second (bps).
pub enum DataRate {
    /// Represents 250 Kbps.
    Low,
    /// Represents 1 Mbps.
    Med,
    /// Represents 2 Mbps.
    High,
}

impl DataRate {
    fn bv(&self) -> u8 {
        match self {
            DataRate::Low  => 0x03,
            DataRate::Med  => 0x00,
            DataRate::High => 0x05,
        }
    }

    fn mask(&self) -> u8 {
        1 << self.bv()
    }
}

enum CRCLength {
    /// No CRC checksum.
    CRCDisabled,
    /// 8-bit CRC checksum.
    CRC8,
    /// 16-bit CRC checksum.
    CRC16,
}

pub struct RF24 {
    status: u8,
    ce_pin: Pin,
    csn_pin: Pin,
    spi_speed: u32,
    payload_size: u8,
    pipe0_reading_address: [u8; 5],
    config_reg: u8, 
    dynamic_payloads_enabled: bool,
    tx_delay: u32,
    cs_delay: u32,
}

impl RF24 {
    pub fn new(csn: Pin, ce: Pin) -> Self {
        Self {
            status: 0,
            ce_pin: ce,
            csn_pin: csn,
            spi_speed: RF24_SPI_SPEED,
            payload_size: 32,
            pipe0_reading_address: [0; 5],
            config_reg: 0,
            dynamic_payloads_enabled: true,
            tx_delay: 0,
            cs_delay: 5,
        }
    }

    pub fn new_with_speed(csn: Pin, ce: Pin, speed: u32) -> Self {
        Self {
            status: 0,
            ce_pin: ce,
            csn_pin: csn,
            spi_speed: if speed < 35_000 { RF24_SPI_SPEED } else { speed },
            payload_size: 32,
            pipe0_reading_address: [0; 5],
            config_reg: 0,
            dynamic_payloads_enabled: true,
            tx_delay: 0,
            cs_delay: 5,
        }
    }

    pub fn begin_reading_pipe(&mut self, address: &str) {
        // Set address to byte array of fixed length.
        self.pipe0_reading_address.copy_from_slice(&address.as_bytes()[0..5]);
    }

    pub fn csn(mode: bool) {

    }

    fn read_register(&mut self, reg: u8) -> u8 {
        spi::begin_transaction(SPISettings::default());

        self.status = spi::transfer(R_REGISTER | reg);
        let result = spi::transfer(0xFF);

        spi::end_transaction();

        result
    }

    fn read_all<const SIZE: usize>(&mut self, reg: u8, len: usize) -> Buffer<u8, SIZE> {
        spi::begin_transaction(SPISettings::default());
        let mut out = buf![];

        self.status = spi::transfer(R_REGISTER | reg);
        for _ in 0..len {
            out.write(spi::transfer(0xFF))
        }

        out
    }

    fn write_register(&mut self, reg: u8, data: u8) {
        spi::begin_transaction(SPISettings::default());

        self.status = spi::transfer(W_REGISTER | reg);
        spi::transfer(data);
        
        spi::end_transaction();
    }
    
    fn write_all<const SIZE: usize>(&mut self, reg: u8, buf: Buffer<u8, SIZE>) {
        spi::begin_transaction(SPISettings::default());

        self.status = spi::transfer(W_REGISTER | reg);
        for byte in buf {
            spi::transfer(byte);
        }

        spi::end_transaction();
    }


}
