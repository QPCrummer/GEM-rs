use core::time::Duration;
use ufmt::uwrite;
use heapless::String;

use panic_probe as _;

/// Preferences defines the consumer-selected range of acceptable values for each category.
/// temperature: The acceptable temperature range in Fahrenheit
/// humidity: The acceptable relative humidity percentage range
/// date: The current date and time: Sec, Min, Hour, Day, Month, Year
/// watering: The minute and hour range for when watering should occur
pub struct Preferences {
    pub temperature: (u8, u8),
    pub humidity: (u8, u8),
    pub date: (u8, u8, u8, u8, u8, u16), // Sec, Min, Hour, Day, Month, Year
    pub watering: Option<(u8, u8, u8, u8)>, // Start (Min, Hour), End (Min, Hour)
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            temperature: (60, 80),       // Ideal range is 60F - 80F
            humidity: (60, 70),          // Ideal range is 60% - 70%
            date: (0, 0, 0, 0, 0, 2000), // Date: 00:00:00 Jan 1 2000
            watering: None,              // No default watering times set
        }
    }
}

impl Preferences {
    // TODO Use time instants to better track time
    /// Increments by 1 second
    pub fn tick_time(&mut self) {
        self.date.0 += 1;

        // Check for rollovers
        if self.date.0 >= 60 {
            self.date.1 += self.date.0 / 60;
            self.date.0 %= 60;
        } else {
            return;
        }

        if self.date.1 >= 60 {
            self.date.2 += self.date.1 / 60;
            self.date.1 %= 60;
        } else {
            return;
        }

        if self.date.2 >= 24 {
            self.date.3 += self.date.2 / 24;
            self.date.2 %= 24;
        } else {
            return;
        }

        // Handle month and day rollovers
        loop {
            let days_in_month = self.get_days_in_month();

            if self.date.3 > days_in_month {
                self.date.3 -= days_in_month;
                self.date.4 += 1;
            } else {
                break;
            }

            if self.date.4 > 12 {
                self.date.4 = 1;
                self.date.5 += 1;
            }
        }

        // Update the date tuple
        self.date = (
            self.date.0,
            self.date.1,
            self.date.2,
            self.date.3,
            self.date.4,
            self.date.5,
        );
    }

    /// Gets the date in the HH:MM:SS DD/MM/YYYY format
    /// Since the indexes start at 0 and months and days start at 1,
    /// the function ensures that 1 is added
    /// returns: (HH:MM:SS, DD/MM/YYYY)
    pub fn get_date_formatted(&mut self) -> (String<8>, String<10>) {
        // Format the date as a string
        let mut val1: String<8> = String::new();
        let mut val2: String<10> = String::new();
        // Format time
        uwrite!(
        &mut val1,
        "{}:{}:{}",
        Self::pad_number(self.date.2),
        Self::pad_number(self.date.1),
        Self::pad_number(self.date.0)
        ).unwrap();

        // Format date
        uwrite!(
        &mut val2,
        "{}/{}/{}",
        Self::pad_number(self.date.3 + 1),
        Self::pad_number(self.date.4 + 1),
        self.date.5
        ).unwrap();

        (val1, val2)
    }

    /// Pads a number with a zero before it if < 10
    /// NOTE: Only supports values <100
    /// param num: number to be padded
    /// returns: String with formatted value
    fn pad_number(num: u8) -> String<2> {
        let mut padded = String::new();
        if num < 10 {
            uwrite!(padded, "0{}", num).unwrap();
        } else {
            uwrite!(padded, "{}", num).unwrap();
        }
        padded
    }

    /// Calculates if it is leap year
    /// param year: The current year
    fn is_leap_year(year: u16) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Gets the next index for the current day depending on the month and leap year
    /// param increment: If the values are incrementing (not decrementing)
    /// returns the next day's index
    pub fn change_days(&self, increment: bool) -> u8 {
        let days_in_month: u8 = self.get_days_in_month();

        if increment {
            (self.date.3 + 1) % days_in_month
        } else {
            (self.date.3 + (days_in_month - 1)) % days_in_month
        }
    }

    /// Gets the amount of days in the current month
    /// returns the amount of days in the month
    fn get_days_in_month(&self) -> u8 {
        match self.date.4 {
            2 => {
                if Self::is_leap_year(self.date.5) {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// Checks if it is time to enable the sprinklers
    /// returns if the current time is within the watering time
    /// returns false if there is no watering time set
    pub fn is_watering_time(&self) -> bool {
        if let Some(watering_time) = self.watering {
            self.date.1 >= watering_time.0 && // Minutes are not too small
                self.date.1 <= watering_time.2 && // Minutes are not too large
                self.date.2 >= watering_time.1 && // Hours are not too small
                self.date.2 <= watering_time.3 // Hours are not too large
        } else {
            false
        }
    }

    /// Formats the watering time: HH:MM - HH:MM
    /// Returns a String of length 16 containing the formatted times
    pub fn format_watering_time(&self) -> String<16> {
        let mut str: String<16> = String::new();
        if let Some(watering_time) = self.watering {
            uwrite!(
                str,
                "{}:{} - {}:{}",
                Self::pad_number(watering_time.1),
                Self::pad_number(watering_time.0),
                Self::pad_number(watering_time.3),
                Self::pad_number(watering_time.2)
            )
                .unwrap();
        } else {
            uwrite!(str, "None").unwrap();
        }
        str
    }

    /// Sets the watering time from 00:00 to 01:00
    pub fn set_default_watering_time(&mut self) {
        self.watering = Some((0, 0, 0, 1));
    }
}