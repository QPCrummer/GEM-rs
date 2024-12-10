use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use hd44780_driver::bus::FourBitBus;
use hd44780_driver::charset::{CharsetUniversal, EmptyFallback};
use hd44780_driver::HD44780;
use hd44780_driver::memory_map::StandardMemoryMap;
use heapless::String;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1, Gpio10, Gpio11, Gpio12, Gpio2, Gpio3, Gpio4, Gpio5};
use rp_pico::hal::gpio::{FunctionSio, Pin, PullDown, SioInput, SioOutput};
use rp_pico::hal::Timer;
use ufmt::uwrite;
use crate::preferences::Preferences;

use panic_probe as _;

type Lcd =  HD44780<FourBitBus<Pin<Gpio0, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio1, FunctionSio<SioOutput>, PullDown>, Pin<Gpio2, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio3, FunctionSio<SioOutput>, PullDown>, Pin<Gpio4, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio5, FunctionSio<SioOutput>, PullDown>>, StandardMemoryMap<16, 2>, EmptyFallback<CharsetUniversal>>;

/// Basic function for rendering text onto the LCD
/// It only clears the screen when the top line is written to
/// param line: text to render
/// param top_line: if the top line is to be written to
/// param lcd: LCD instance
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

/// Renders the Preferences on screen with a blinking indicator cursor
/// param line: The preferences line
/// param left_cursor: If the lower bound is selected
/// param lcd: LCD instance
pub fn render_edit_screen<const N: usize>(line: &String<N>, left_cursor: bool, lcd: &mut Lcd, delay: &mut Timer) {
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

/// Renders the current date unit (min, hr, day, etc.) on the first line with a central blinking cursor on the second line
/// param line: The date line
/// param lcd: LCD instance
pub fn render_date_edit_screen<const N: usize>(line: &String<N>, lcd: &mut Lcd, delay: &mut Timer) {
    // Clear
    lcd.clear(delay).unwrap();

    // Write date segment
    lcd.set_cursor_pos(0, delay).unwrap();
    lcd.write_str(line, delay).unwrap();

    // Create selection cursor
    render_selector(true, 7, lcd, delay);
}

/// Renders a ■ on the bottom line at the specified position
/// param active: whether to add or remove a ■
/// param bottom_pos: the x-coordinate
/// param lcd: LCD instance
pub fn render_selector(active: bool, bottom_pos: u8, lcd: &mut Lcd, delay: &mut Timer) {
    lcd.set_cursor_xy((bottom_pos, 1), delay).unwrap();
    if active {
        lcd.write_char('█', delay).unwrap();
    } else {
        lcd.write_char(' ', delay).unwrap();
    }
}

/// Renders configuration screens for various parts of the date system
/// param unit: The current unit; Ex: Minutes
/// param info_str: String<N> for data
/// param modulo: The range for the unit; Ex: 60 for Minutes
/// param preference: Current variable being assigned
/// param preferences: Preferences instance
/// param lcd: LCD instance
/// param delay: Delay instance
/// param up_button: Up button instance
/// param down_button: Down button instance
/// param select_button: Select button instance
pub fn render_time_config_screen(
    unit: &str,
    info_str: &mut String<11>,
    modulo: u16,
    preference: &mut u16,
    preferences: &mut Preferences,
    lcd: &mut Lcd,
    delay: &mut Timer,
    up_button: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    down_button: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    select_button: &mut Pin<Gpio12, FunctionSio<SioInput>, PullDown>,
)
{
    let mut refresh: bool = true;
    let mut update_date: bool = false;
    loop {
        if refresh {
            uwrite!(info_str, "{}: {}", unit, preference)
                .unwrap(); // Max str size 10
            render_date_edit_screen(&info_str, lcd, delay);
            refresh = false;
        }

        delay.delay_ms(500);

        if update_date {
            preferences.tick_time();
        }
        update_date = !update_date;

        if up_button.is_high().unwrap() {
            *preference = (*preference + 1) % modulo;
            refresh = true;
        } else if down_button.is_high().unwrap() {
            *preference = (*preference + (modulo - 1)) % modulo;
            refresh = true;
        } else if select_button.is_high().unwrap() {
            break;
        }
    }
}