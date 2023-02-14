//! Low level time and date functions.

use crate::constants::TIME;
use crate::timing::millis;

/// An approximation of the current time. Use an RTC for a more accurate measurement.
pub fn now() -> DateTime {
    DateTime::from_unix(unix())
}

/// Approximation of the current unix time. Use an RTC for a more accurate measurement.
pub fn unix() -> u64 {
    TIME + millis()/1000
}

/// Combined date and time in the GMT time zone.
pub struct DateTime {
    /// Starts at year 0.
    pub year: usize,
    /// The month.
    pub month: Month,
    /// Will be between 0-30.
    pub day: u8,
    /// The day of the week. Starts at Sunday.
    pub weekday: Weekday,
    /// Will be between 0-23.
    pub hour: u8,
    /// Will be between 0-59.
    pub minute: u8,
    /// Will be between 0-59.
    pub second: u8,
}

impl DateTime {
    /// Creates `Time` from a unix timestamp (in seconds).
    pub fn from_unix(time: u64) -> DateTime {
        

        let second = time % 60;

        let minutes = time / 60; // Convert time to minutes.
        let minute = minutes % 60;

        let hours = minutes / 60; // Convert time to hours.
        let hour = hours % 24;

        let days = (hours/24) as usize;
        let weekday = (days + 4) % 7; // Unix epoch is a thursday

        crate::prelude::println!("{}", days);

        let year = (((days as u64 * 4) / 1461) + 1970) as usize; // days/325.25 + 1970: Accounts for leap years and the fact that Unix time starts at 1970.
        let is_leap_year = leap_year(year);
        
        crate::prelude::println!("{}, {}", days*4, (days*4)/1461);
        
        let leap_days = leap_years_between(1970, year as usize);
        let doy = (days - ((year-1970)*365)) - leap_days;

        let month = Month::from_day(doy, is_leap_year);
        let day = doy - month.days_before(is_leap_year);

        DateTime {
            year: year as usize,
            month,
            day: day as u8,
            weekday: Weekday::from_index(weekday),
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }

    /// Returns the time in seconds from the unix epoch.
    pub fn to_unix(&self) -> u64 {
        let month_seconds = self.month.days_before(leap_year(self.year)) as u64 * 24 * 60 * 60;
        let day_seconds = self.day as u64 * 24 * 60 * 60;

        let hour_seconds = self.hour as u64 * 60 * 60;
        let minute_seconds = self.minute as u64 * 60;
        
        let days_before = (self.year - 1970) as u64 * 365 + leap_years_between(1970, self.year) as u64;
        let year_seconds = days_before *24 * 60 * 60;

        year_seconds + month_seconds + day_seconds + hour_seconds + minute_seconds + self.second as u64
    }
}

impl core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}/{} {}:{}:{}", self.day+1, self.month as u8 + 1, self.year, self.hour, self.minute, self.second)
    }
}

/// Month of the year.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    /// Matches month number to month.
    /// Index is between 0-11.
    pub fn from_index(month: usize) -> Month {
        use Month::*;
        // Keep between 0-11 to avoid panics.
        match month % 12 {
            0  => January,
            1  => February,
            2  => March,
            3  => April,
            4  => May,
            5  => June,
            6  => July,
            7  => August,
            8  => September,
            9  => October,
            10 => November,
            11 => December,
            _  => unreachable!(),
        }
    }

    /// Returns the month a given day is in.
    /// 
    /// Day must be between 0-364, or 365 if it is a leap year. 
    pub fn from_day(day: usize, leap_year: bool) -> Month {
        use Month::*;
        if leap_year {
            match day % 365 {
                0..31    => January,
                31..59   => February,
                59..90   => March,
                90..120  => April,
                120..151 => May,
                151..181 => June,
                181..212 => July,
                212..243 => August,
                243..273 => September,
                273..304 => October,
                304..334 => November,
                334..365 => December,
                _ => unreachable!(),
            }
        } else {
            match day % 366 {
                0..31    => January,
                31..60   => February,
                60..91   => March,
                91..121  => April,
                121..152 => May,
                152..182 => June,
                182..213 => July,
                213..244 => August,
                244..274 => September,
                274..305 => October,
                305..335 => November,
                335..366 => December,
                _ => unreachable!(),
            }
        }
    }

    /// Returns the days in a given month.
    pub fn days(&self, leap_year: bool) -> u8 {
        use Month::*;
        match self {
            January => 31,
            February => if leap_year { 29 } else { 28 },
            March => 31,
            April => 30,
            May => 31,
            June => 30,
            July => 31,
            August => 31,
            September => 30,
            October => 31,
            November => 30,
            December => 31,
        }
    }

    /// Returns the days in the year before the month.
    pub fn days_before(&self, leap_year: bool) -> usize {
        use Month::*;
        let offset = if leap_year && *self as u8 >= 2 { 1 } else { 0 };
        offset + match self {
            January => 0,
            February => 31,
            March => 59,
            April => 90,
            May => 120,
            June => 151,
            July => 181,
            August => 212,
            September => 243,
            October => 273,
            November => 304,
            December => 334,
        }
    }
}

/// The day of the week
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    /// Returns the weekday at the provided weekday number.
    pub fn from_index(day: usize) -> Weekday {
        use Weekday::*;
        // Keep within 0-6 to avoid panics.
        match day % 7 {
            0 => Sunday,
            1 => Monday,
            2 => Tuesday,
            3 => Wednesday,
            4 => Thursday,
            5 => Friday,
            6 => Saturday,
            _ => unreachable!(),
        }
    }
}

/// Returns `true` if the given year is a leap year.
pub fn leap_year(year: usize) -> bool {
    year%4 == 0 && ( year%100 > 0 || year%400 == 0 )
}

/// The number of leap years between the given year and year 0.
pub fn leap_years_before(year: usize) -> usize {
    let year_before = year-1;
    (year_before/4) - (year_before/100) + (year_before/400)
}

/// The number of leap years between the given years.
pub fn leap_years_between(start: usize, end: usize) -> usize {
    let before = start.min(end);
    let after = end.max(start);
    leap_years_before(after) - leap_years_before(before + 1)
}
