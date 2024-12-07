use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use heapless::String;
use lcd1602_rs::LCD1602;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1, Gpio10, Gpio11, Gpio12, Gpio2, Gpio3, Gpio4, Gpio5};
use rp_pico::hal::gpio::{FunctionSio, Pin, PullDown, PullUp, SioInput, SioOutput};
use rp_pico::hal::Timer;
use ufmt::uwrite;
use crate::preferences::Preferences;

use panic_probe as _;

type Lcd = LCD1602<
    Pin<Gpio1, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio0, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio2, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio3, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio4, FunctionSio<SioOutput>, PullDown>,
    Pin<Gpio5, FunctionSio<SioOutput>, PullDown>,
    Timer,
>;

/// Basic function for rendering text onto the LCD
/// It only clears the screen when the top line is written to
/// param line: text to render
/// param top_line: if the top line is to be written to
/// param lcd: LCD instance
pub fn render_screen(line: &str, top_line: bool, lcd: &mut Lcd) {
    // Set cursor to the correct line
    if top_line {
        // Reset screen
        lcd.clear().unwrap();
        lcd.set_position(0, 0).unwrap();
    } else {
        lcd.set_position(0, 1).unwrap();
    }
    lcd.print(line).unwrap();
}

/// Renders the Preferences on screen with a blinking indicator cursor
/// param line: The preferences line
/// param left_cursor: If the lower bound is selected
/// param lcd: LCD instance
pub fn render_edit_screen<const N: usize>(line: &String<N>, left_cursor: bool, lcd: &mut Lcd) {
    // Clear
    lcd.clear().unwrap();

    // Write top info
    lcd.set_position(0, 0).unwrap();
    lcd.print(line).unwrap();

    // Create selection cursor
    if left_cursor {
        render_selector(true, 0, lcd);
    } else {
        render_selector(false, 0, lcd);
        render_selector(true, 15, lcd);
    }
}

/// Renders the current date unit (min, hr, day, etc.) on the first line with a central blinking cursor on the second line
/// param line: The date line
/// param lcd: LCD instance
pub fn render_date_edit_screen<const N: usize>(line: &String<N>, lcd: &mut Lcd) {
    // Clear
    lcd.clear().unwrap();

    // Write date segment
    lcd.set_position(0, 0).unwrap();
    lcd.print(line).unwrap();

    // Create selection cursor
    render_selector(true, 7, lcd);
}

/// Renders a ■ on the bottom line at the specified position
/// param active: whether to add or remove a ■
/// param bottom_pos: the x-coordinate
/// param lcd: LCD instance
pub fn render_selector(active: bool, bottom_pos: u8, lcd: &mut Lcd) {
    lcd.set_position(bottom_pos, 1).unwrap();
    if active {
        lcd.print("■").unwrap();
    } else {
        lcd.print(" ").unwrap();
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
            render_date_edit_screen(&info_str, lcd);
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