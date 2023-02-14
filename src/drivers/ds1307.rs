//! Used to interface with the DS1307 RTC.

use crate::libraries::wire;
use crate::bits::{ from_bcd, from_dec };
use crate::time::{ DateTime, Weekday, Month };

const DS1307_ADDRESS: usize = 0x68;

/// There are 7 data fields (secs, min, hr, dow, date, mth, yr)
const FIELDS: u8 = 7;

pub enum Error {
    NotExist,
    RequestFailed,
    Halted,
}

pub fn read() -> Result<DateTime, Error> {
    wire::write(0x00);

    if wire.end_transmission.is_err() {
        return Err(Error::NotExist);
    }

    let req = wire::request_from(DS1307_ADDRESS, FIELDS, true);
    if wire::available() < FIELDS || req.is_err() {
        return Err(Error::RequestFailed);
    }
    
    let sec = wire::read();
    let second = from_bcd(sec & 0x7F);
    let minute = from_bcd(wire::read());
    let hour = from_bcd(wire::read() & 0x3F); // Mask assumes a 24hr clock
    let weekday = from_bcd(wire::read());
    let day = from_bcd(wire::read());
    let month = from_bcd(wire::read());
    let year = from_bcd(wire::read());

    if bits::read(sec, 7) {
        return Err(Error::Halted);
    }

    Ok(
        DateTime {
            second,
            minute,
            hour: hour-1,
            weekday: Weekday::from_index(weekday-1),
            day: day-1,
            month: Month::from_index(month-1),
            year,
        }
    )
}
