use crate::preferences::{inclusive_iterator, Preferences};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use hd44780_driver::bus::FourBitBus;
use hd44780_driver::charset::{CharsetUniversal, EmptyFallback};
use hd44780_driver::memory_map::StandardMemoryMap;
use hd44780_driver::HD44780;
use heapless::String;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1, Gpio10, Gpio11, Gpio12, Gpio2, Gpio3, Gpio4, Gpio5};
use rp_pico::hal::gpio::{FunctionSio, Pin, PullDown, SioInput, SioOutput};
use rp_pico::hal::Timer;
use ufmt::uwrite;

use panic_probe as _;

pub type Lcd = HD44780<
    FourBitBus<
        Pin<Gpio0, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio1, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio2, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio3, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio4, FunctionSio<SioOutput>, PullDown>,
        Pin<Gpio5, FunctionSio<SioOutput>, PullDown>,
    >,
    StandardMemoryMap<16, 2>,
    EmptyFallback<CharsetUniversal>,
>;

/// Basic function for rendering text onto the LCD.
/// It only clears the screen when the top line is written to
///
/// - param line: text to render
/// - param top_line: if the top line is to be written to
/// - param lcd: [Lcd] instance
pub fn render_screen(line: &str, top_line: bool, lcd: &mut Lcd, delay: &mut Timer) {
    // Set cursor to the correct line
    if top_line {
        // Reset screen
        lcd.clear(delay).unwrap();
        lcd.set_cursor_pos(0, delay).unwrap();
    } else {
        lcd.set_cursor_xy((0, 1), delay).unwrap();
    }
    lcd.write_str(line, delay).unwrap();
}

/// Renders the Preferences on screen with a `^` cursor
///
/// - param line: The preferences line
/// - param left_cursor: If the lower bound is selected
/// - param lcd: [Lcd] instance
/// - param delay: [Timer] instance
pub fn render_edit_screen<const N: usize>(
    line: &String<N>,
    left_cursor: bool,
    lcd: &mut Lcd,
    delay: &mut Timer,
) {
    // Clear
    lcd.clear(delay).unwrap();

    // Write top info
    lcd.set_cursor_pos(0, delay).unwrap();
    lcd.write_str(line, delay).unwrap();

    // Create selection cursor
    if left_cursor {
        render_selector(true, 0, lcd, delay);
    } else {
        render_selector(false, 0, lcd, delay);
        render_selector(true, 15, lcd, delay);
    }
}

/// Renders the Preferences watering editing screen with a `^` cursor
///
/// - param line: The preferences line
/// - param index: If index of the element being edited
/// - param lcd: [Lcd] instance
/// - param delay: Timer instance
pub fn render_watering_edit_screen<const N: usize>(
    line: &String<N>,
    index: i32,
    lcd: &mut Lcd,
    delay: &mut Timer,
) {
    // Clear
    lcd.clear(delay).unwrap();

    // Write top info
    lcd.set_cursor_pos(0, delay).unwrap();
    lcd.write_str(line, delay).unwrap();

    // Create selection cursor
    match index {
        1 => {
            render_selector(false, 0, lcd, delay);
            render_selector(true, 3, lcd, delay);
        }
        0 => {
            render_selector(true, 0, lcd, delay);
        }
        2 => {
            render_selector(false, 3, lcd, delay);
            render_selector(true, 8, lcd, delay);
        }
        _ => {
            render_selector(false, 8, lcd, delay);
            render_selector(true, 11, lcd, delay);
        }
    }
}

/// Renders the current date unit `(min, hr, day, etc.)` on the first line with a `^` cursor on the second line
///
/// - param line: The date line
/// - param lcd: [Lcd] instance
pub fn render_date_edit_screen<const N: usize>(line: &String<N>, lcd: &mut Lcd, delay: &mut Timer) {
    // Clear
    lcd.clear(delay).unwrap();

    // Write date segment
    lcd.set_cursor_pos(0, delay).unwrap();
    lcd.write_str(line, delay).unwrap();

    // Create selection cursor
    render_selector(true, 7, lcd, delay);
}

/// Renders a `^` on the bottom line at the specified position
///
/// - param active: whether to add a `^`
/// - param bottom_pos: the x-coordinate on the bottom row
/// - param lcd: [Lcd] instance
pub fn render_selector(active: bool, bottom_pos: u8, lcd: &mut Lcd, delay: &mut Timer) {
    lcd.set_cursor_xy((bottom_pos, 1), delay).unwrap();
    if active {
        lcd.write_str("^", delay).unwrap();
    } else {
        lcd.write_str(" ", delay).unwrap();
    }
}

/// Renders configuration screens for various parts of the date system
///
/// - param unit: The current unit; Ex: Minutes
/// - param info_str: [String] for data
/// - param min: The minimum value for the unit
/// - param max: The maximum value for the unit
/// - param preference: Current variable being assigned
/// - param preferences: [Preferences] instance
/// - param lcd: [Lcd] instance
/// - param delay: [Timer] instance
/// - param up_button: Up button instance
/// - param down_button: Down button instance
/// - param select_button: Select button instance
///
/// returns the inputted preference value after modification
///
/// ## Example:
/// ```rust
/// use rp_pico::hal::Timer;
/// use greenhouse_rs::preferences::Preferences;
/// use greenhouse_rs::rendering::{render_time_config_screen, Lcd};
///
/// let mut preferences = Preferences::default();
/// let mut info_str: heapless::String<11>; // Must be a heapless String with size 11
/// let mut lcd: Lcd;
/// let mut delay: Timer;
/// let mut up_button;     // GPIO
/// let mut down_button;   // GPIO
/// let mut select_button; // GPIO
///
/// preferences.date.1 = render_time_config_screen( // Set the Minutes to the return value
///     "Minute",           // Name of the unit is "Minute"
///     &mut info_str,
///     0,                  // The minimum minute value is 0
///     59,                 // The maximum minute value is 59
///     preferences.date.1, // Pass the minute variable
///     &mut preferences,
///     &mut lcd,
///     &mut delay,
///     &mut up_button,
///     &mut down_button,
///     &mut select_button,
///  );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn render_time_config_screen(
    unit: &str,
    info_str: &mut String<11>,
    min: u8,
    max: u8,
    mut preference: u8,
    preferences: &mut Preferences,
    lcd: &mut Lcd,
    delay: &mut Timer,
    up_button: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    down_button: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    select_button: &mut Pin<Gpio12, FunctionSio<SioInput>, PullDown>,
) -> u8 {
    let mut refresh: bool = true;
    let mut update_date: bool = false;
    loop {
        if refresh {
            uwrite!(info_str, "{}: {}", unit, preference).unwrap();
            render_date_edit_screen(info_str, lcd, delay);
            info_str.clear();
            refresh = false;
        }

        delay.delay_ms(500);

        if update_date {
            preferences.tick_time();
        }
        update_date = !update_date;

        if up_button.is_high().unwrap() {
            preference = inclusive_iterator(preference, min, max, true);
            refresh = true;
        } else if down_button.is_high().unwrap() {
            preference = inclusive_iterator(preference, min, max, false);
            refresh = true;
        } else if select_button.is_high().unwrap() {
            break;
        }
    }
    preference
}
