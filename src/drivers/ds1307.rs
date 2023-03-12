//! Used to interface with the DS1307 RTC.
//! 
//! This is a port of [DS1307RTC](https://github.com/PaulStoffregen/DS1307RTC)

use crate::libraries::wire;
use crate::bits;
use crate::libraries::time::{ DateTime, Weekday, Month };

const DS1307_ADDRESS: u8 = 0x68;

/// There are 7 data fields (secs, min, hr, dow, date, mth, yr)
const FIELDS: usize = 7;

/// Various I2C errors that can occur while interfacing with the DS1307.
#[derive(Debug)]
pub enum Error {
    /// This error occurs when the DS1307 does not exist or is not connected to the I2C bus.
    NotExist,
    /// This error occurs when requested data is not recieved, or when reading data fails.
    RequestFailed,
    /// This error occurs when the clock on the DS1307 is stopped.
    Halted,
    /// This error occurs when data is not able to be written.
    WriteFail,
}

/// Gets the current time as a Unix timestamp
pub fn unix() -> Result<u64, Error> {
    Ok(read()?.to_unix())
}

/// Read the current time from the DS1307
pub fn read() -> Result<DateTime, Error> {
    wire::begin_transmission(DS1307_ADDRESS);
    wire::write(0x00).map_err(|_| Error::WriteFail)?;
    if wire::end_transmission(true).is_err() {
        return Err(Error::NotExist);
    }
    
    let req = wire::request_from(DS1307_ADDRESS, FIELDS as u8, true);
    if wire::available() < FIELDS || req.is_err() {
        return Err(Error::RequestFailed);
    }
    
    let sec = wire::read().ok_or(Error::RequestFailed)?;
    let second = bits::from_bcd(sec & 0x7F);
    let minute = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)?);
    let hour = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)? & 0x3F); // Mask assumes a 24hr clock
    let weekday = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)?);
    let day = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)?);
    let month = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)?);
    let year = bits::from_bcd(wire::read().ok_or(Error::RequestFailed)?);

    if bits::read(sec, 7) {
        return Err(Error::Halted);
    }

    Ok(
        DateTime {
            second,
            minute,
            hour: hour-1,
            weekday: Weekday::from_index((weekday-1) as usize),
            day: day-1,
            month: Month::from_index((month-1) as usize),
            year: year as usize + 2000, // Offset is from 2000 (Y2k)
        }
    )
}

/// Set the time stored in the DS1307
pub fn write(date: DateTime) -> Result<(), Error> {
    // To eliminate any potential race condition,
    // stop the clock before writing the values,
    // then restart it after
    wire::begin_transmission(DS1307_ADDRESS);

    wire::write(0x00).map_err(|_| Error::WriteFail)?; // Reset register pointer
    wire::write(0x80).map_err(|_| Error::WriteFail)?; // Stop the clock. The seconds will be written last

    wire::write(bits::from_dec(date.minute)).map_err(|_| Error::WriteFail)?;
    wire::write(bits::from_dec(date.hour)).map_err(|_| Error::WriteFail)?; // Sets the 24 hour format
    wire::write((date.weekday as u8)+1).map_err(|_| Error::WriteFail)?;
    wire::write((date.day)+1).map_err(|_| Error::WriteFail)?;
    wire::write((date.month as u8)+1).map_err(|_| Error::WriteFail)?;
    wire::write((date.year- 2000) as u8).map_err(|_| Error::WriteFail)?;

    wire::end_transmission(true).map_err(|_| Error::NotExist)?;

    // Now go back and set the seconds, starting the clock back up as a side effect.
    wire::begin_transmission(DS1307_ADDRESS);
    wire::write(0x00).map_err(|_| Error::WriteFail)?;
    wire::write(date.second).map_err(|_| Error::WriteFail)?;

    wire::end_transmission(true).map_err(|_| Error::NotExist)?;

    Ok(())
}

/// Returns whether or not the clock in the DS1307 is running.
pub fn is_running() -> Result<bool, Error> {
    wire::begin_transmission(DS1307_ADDRESS);
    wire::write(0x00).map_err(|_| Error::WriteFail)?;
    wire::end_transmission(true).map_err(|_| Error::NotExist)?;

    // Just fetch the seconds register and check the top bit
    wire::request_from(DS1307_ADDRESS, 1, true).map_err(|_| Error::RequestFailed)?;
    let sec = wire::read().ok_or(Error::RequestFailed)?;

    Ok(bits::read(sec, 7))
}
