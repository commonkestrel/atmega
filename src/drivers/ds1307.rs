//! Used to interface with the DS1307 RTC.

use crate::libraries::{ time, wire };
use crate::bits::{ from_bcd, from_dec };

const DS1307_ADDRESS: usize = 0x68;

/// There are 7 data fields (secs, min, hr, dow, date, mth, yr)
const FIELDS: u8 = 7;

pub enum Error {
    NotExist,
    RequestFailed,
}

pub fn read() -> Result<time::DateTime, Error> {
    wire::write(0x00);

    if wire.end_transmission.is_err() {
        return Err(Error::NotExist);
    }

    wire::request_from(DS1307_ADDRESS, FIELDS, true);
    if wire::available() < FIELDS {
        return Err(Error::RequestFailed);
    }
}
