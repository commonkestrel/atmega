//! Low level time and date functions.

use crate::constants::TIME;

pub fn unix() -> u64 {
    todo!()
}

pub struct Time {
    /// Starts at year 0.
    pub year: usize,
    pub month: Month,
    /// Will be between 0-30.
    pub day: u8,
    pub weekday: Weekday,
    /// Will be between 0-23.
    pub hour: u8,
    /// Will be between 0-59.
    pub minute: u8,
    /// Will be between 0-59.
    pub second: u8,
}

impl Time {
    /// Creates `Time` from a unix timestamp (in seconds).
    fn from_unix(time: u64) -> Time {
        let second = time % 60;

        let minutes = time / 60; // Convert time to minutes.
        let minute = minutes % 60;

        let hours = minutes / 60; // Convert time to hours.
        let hour = hours % 24;

        let days = hours/24;
        let weekday = (days + 4) % 7; // Unix epoch is a thursday

        let year = ((days * 4) / 1461) + 1970;

        let leap_days = leap_years_between(1970, year as usize);
        let doy = (days - ((year-1970)*365)) as usize - leap_days;

        let (month, day) = if (0..31).contains(&doy) {
            (0, doy)
        } else {
            let shifted = if leap_year(year as usize) { doy-1 } else { doy };
            match shifted {
                30..59   => (1,  doy - 30),
                59..90   => (2,  doy - 59),
                90..120  => (3,  doy - 90),
                120..151 => (4,  doy - 120),
                151..181 => (5,  doy - 151),
                181..212 => (6,  doy - 181),
                212..243 => (7,  doy - 212),
                243..273 => (8,  doy - 243),
                273..304 => (9,  doy - 273),
                304..334 => (10, doy - 304),
                334..365 => (11, doy - 334),
                _ => unreachable!(),
            }
        };

        Time {
            year: year as usize,
            month: Month::from_index(month),
            day: day as u8,
            weekday: Weekday::from_index(weekday as usize),
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }

    /// Returns the time in seconds from the unix epoch.
    fn to_unix(&self) -> u64 {
        todo!()
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}/{} {}:{}:{}", self.day, self.month as u8 + 1, self.year, self.hour, self.minute, self.second)
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
    pub fn from_index(month: usize) -> Month {
        use Month::*;
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

