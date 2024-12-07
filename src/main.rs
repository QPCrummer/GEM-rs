#![no_std]
#![no_main]

use bme680::{
    Bme680, FieldData, I2CAddress, IIRFilterSize, OversamplingSetting, PowerMode, SettingsBuilder,
};
use bsp::entry;
use core::time::Duration;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::digital::OutputPin;
use embedded_hal::digital::StatefulOutputPin;
use panic_probe as _;
use rp_pico::hal::Timer;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    watchdog::Watchdog,
};
use greenhouse_rs::preferences::Preferences;
use greenhouse_rs::rendering::{
    render_date_edit_screen, render_edit_screen, render_screen, render_selector,
    render_time_config_screen,
};
use greenhouse_rs::sensors::{get_bme_data, get_humidity, get_pressure, get_temperature};
use heapless::String;
use i2c_pio::I2C;
use lcd1602_rs::LCD1602;
use rp_pico::hal;
use rp_pico::hal::fugit::RateExtU32;
use rp_pico::hal::gpio::bank0::{Gpio10, Gpio11, Gpio12, Gpio6};
use rp_pico::hal::gpio::{FunctionSio, Pin, PullDown, SioInput, SioOutput};
use rp_pico::hal::pio::PIOExt;
use ufmt::uwrite;

const FIRE: &str = "Fire Present";

#[entry]
fn main() -> ! {
    info!("GreenHousePi Starting");
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let i2c_pio = I2C::new(
        &mut pio,
        pins.gpio8,
        pins.gpio9,
        sm0,
        100.kHz(),
        clocks.system_clock.freq(),
    );

    // Set up BME680
    let mut bme = Bme680::init(i2c_pio, &mut delay, I2CAddress::Secondary).unwrap();
    let settings = SettingsBuilder::new()
        .with_humidity_oversampling(OversamplingSetting::OS2x)
        .with_pressure_oversampling(OversamplingSetting::OS4x)
        .with_temperature_oversampling(OversamplingSetting::OS8x)
        .with_temperature_filter(IIRFilterSize::Size3)
        .with_gas_measurement(Duration::from_millis(1500), 320, 25)
        .with_run_gas(true)
        .build();

    bme.set_sensor_settings(&mut delay, settings).unwrap();

    bme.set_sensor_mode(&mut delay, PowerMode::ForcedMode)
        .unwrap();

    // Set up LCD1602
    let mut lcd = LCD1602::new(
        pins.gpio1.into_function(),
        pins.gpio0.into_function(),
        pins.gpio2.into_function(),
        pins.gpio3.into_function(),
        pins.gpio4.into_function(),
        pins.gpio5.into_function(),
        delay,
    )
    .unwrap();

    // Set up button up
    let mut up_button = pins.gpio10.into_pull_down_input();

    // Set up button down
    let mut down_button = pins.gpio11.into_pull_down_input();

    // Set up button select
    let mut select_button = pins.gpio12.into_pull_down_input();

    // Set up buzzer
    let mut buzzer = pins.gpio6.into_push_pull_output();

    // Set up smoke detector
    let mut smoke_detector = pins.gpio7.into_pull_up_input();

    // Set up sprinklers
    let mut sprinklers = pins.gpio13.into_push_pull_output();

    // Set up roof vent
    let mut roof_vent = pins.gpio14.into_push_pull_output();

    let mut current_screen_index: u8 = 0;
    let mut wait_time: u16 = 0;
    let mut data: FieldData = FieldData::default(); // TODO Make sure this is set to a valid value before using it
    let mut preferences: Preferences = Preferences::default();
    // Cooldowns
    let mut button_cooldown: u8 = 50; // 500ms cooldown

    info!("GreenHousePi Ready");

    loop {
        delay.delay_ms(10);

        // Tick buttons
        button_cooldown = tick_buttons(button_cooldown);

        let (update_needed, action) = should_update(
            &mut up_button,
            &mut down_button,
            &mut select_button,
            &mut wait_time,
            &mut preferences,
        );

        info!("update_needed: {:?}", update_needed);
        info!("action: {:?}", action);

        if update_needed {
            match action {
                RefreshAction::Up => {
                    debug(&mut buzzer, delay);
                    if button_cooldown == 0 {
                        current_screen_index = next_screen(current_screen_index, true);
                        button_cooldown = 50;
                    }
                }
                RefreshAction::Down => {
                    debug(&mut buzzer, delay);
                    debug(&mut buzzer, delay);
                    if button_cooldown == 0 {
                        current_screen_index = next_screen(current_screen_index, false);
                        button_cooldown = 50;
                    }
                }
                RefreshAction::Select => {
                    debug(&mut buzzer, delay);
                    debug(&mut buzzer, delay);
                    debug(&mut buzzer, delay);
                    // Handle SELECT action
                    if button_cooldown == 0 {
                        lcd.clear().unwrap();
                        let mut editing_lower: bool = true;
                        let mut update_date: bool = false;
                        let mut refresh: bool = true;
                        let mut info_str: String<11> = String::new();
                        match current_screen_index {
                            0 => {
                                // Temp
                                for _ in 0..2 {
                                    loop {
                                        if refresh {
                                            uwrite!(
                                                &mut info_str,
                                                "{} - {}",
                                                preferences.temperature.0,
                                                preferences.temperature.1
                                            )
                                            .unwrap(); // Max str size 7
                                            render_edit_screen(&info_str, editing_lower, &mut lcd);
                                            refresh = false;
                                        }

                                        delay.delay_ms(500);

                                        if update_date {
                                            preferences.tick_time();
                                        }
                                        update_date = !update_date;

                                        if up_button.is_high().unwrap() {
                                            if editing_lower {
                                                if preferences.temperature.0 < 1 {
                                                    preferences.temperature.0 += 1;
                                                }
                                            } else if preferences.temperature.1 < 1 {
                                                preferences.temperature.1 += 1;
                                            }
                                            refresh = true;
                                        } else if down_button.is_high().unwrap() {
                                            if editing_lower {
                                                if preferences.temperature.0 > 0 {
                                                    preferences.temperature.0 -= 1;
                                                }
                                            } else if preferences.temperature.1 > 0 {
                                                preferences.temperature.1 -= 1;
                                            }
                                            refresh = true;
                                        } else if select_button.is_high().unwrap() {
                                            editing_lower = false;
                                            render_selector(false, 15, &mut lcd);

                                            refresh = true;
                                            break;
                                        }
                                    }
                                }
                                // Check legality
                                if preferences.temperature.0 > preferences.temperature.1 {
                                    core::mem::swap(
                                        &mut preferences.temperature.0,
                                        &mut preferences.temperature.1,
                                    );
                                }
                            }
                            1 => {
                                // Humidity
                                for _ in 0..2 {
                                    loop {
                                        if refresh {
                                            uwrite!(
                                                &mut info_str,
                                                "{}% - {}%",
                                                preferences.humidity.0,
                                                preferences.humidity.1
                                            )
                                            .unwrap(); // Max str size 11
                                            render_edit_screen(&info_str, editing_lower, &mut lcd);
                                            refresh = false;
                                        }

                                        delay.delay_ms(500);

                                        if update_date {
                                            preferences.tick_time();
                                        }
                                        update_date = !update_date;

                                        if up_button.is_high().unwrap() {
                                            if editing_lower {
                                                if preferences.humidity.0 < 100 {
                                                    preferences.humidity.0 += 1;
                                                }
                                            } else if preferences.humidity.1 < 100 {
                                                preferences.humidity.1 += 1;
                                            }
                                            refresh = true;
                                        } else if down_button.is_high().unwrap() {
                                            if editing_lower {
                                                if preferences.humidity.0 > 0 {
                                                    preferences.humidity.0 -= 1;
                                                }
                                            } else if preferences.humidity.1 > 0 {
                                                preferences.humidity.1 -= 1;
                                            }
                                            refresh = true;
                                        } else if select_button.is_high().unwrap() {
                                            editing_lower = false;
                                            render_selector(false, 15, &mut lcd);
                                            refresh = true;
                                            break;
                                        }
                                    }
                                }
                                // Check legality
                                if preferences.humidity.0 > preferences.humidity.1 {
                                    core::mem::swap(
                                        &mut preferences.humidity.0,
                                        &mut preferences.humidity.1,
                                    );
                                }
                            }
                            3 => {
                                // Date

                                render_time_config_screen(
                                    "Minute",
                                    &mut info_str,
                                    60,
                                    &mut (preferences.date.1 as u16),
                                    &mut preferences,
                                    &mut lcd,
                                    &mut delay,
                                    &mut up_button,
                                    &mut down_button,
                                    &mut select_button,
                                );

                                render_time_config_screen(
                                    "Hour",
                                    &mut info_str,
                                    24,
                                    &mut (preferences.date.2 as u16),
                                    &mut preferences,
                                    &mut lcd,
                                    &mut delay,
                                    &mut up_button,
                                    &mut down_button,
                                    &mut select_button,
                                );

                                // Day
                                loop {
                                    if refresh {
                                        uwrite!(&mut info_str, "Day: {}", preferences.date.3)
                                            .unwrap(); // Max str size 7
                                        render_date_edit_screen(&info_str, &mut lcd);
                                        refresh = false;
                                    }
                                    delay.delay_ms(500);

                                    if update_date {
                                        preferences.tick_time();
                                    }
                                    update_date = !update_date;

                                    if up_button.is_high().unwrap() {
                                        preferences.date.3 = preferences.change_days(true);
                                        refresh = true;
                                    } else if down_button.is_high().unwrap() {
                                        preferences.date.3 = preferences.change_days(false);
                                        refresh = true;
                                    } else if select_button.is_high().unwrap() {
                                        refresh = true;
                                        break;
                                    }
                                }

                                render_time_config_screen(
                                    "Month",
                                    &mut info_str,
                                    12,
                                    &mut (preferences.date.4 as u16),
                                    &mut preferences,
                                    &mut lcd,
                                    &mut delay,
                                    &mut up_button,
                                    &mut down_button,
                                    &mut select_button,
                                );

                                // Year
                                loop {
                                    if refresh {
                                        uwrite!(&mut info_str, "Year: {}", preferences.date.5)
                                            .unwrap(); // Max str size 10
                                        render_date_edit_screen(&info_str, &mut lcd);
                                        refresh = false;
                                    }
                                    delay.delay_ms(500);

                                    if update_date {
                                        preferences.tick_time();
                                    }
                                    update_date = !update_date;

                                    if up_button.is_high().unwrap() {
                                        // I'm going to assume that no one is stupid enough
                                        // to actually hit the u16 integer limit
                                        preferences.date.5 += 1;
                                        refresh = true;
                                    } else if down_button.is_high().unwrap() {
                                        if preferences.date.5 != 0 {
                                            preferences.date.5 -= 1;
                                        }
                                        refresh = true;
                                    } else if select_button.is_high().unwrap() {
                                        break;
                                    }
                                }

                                render_selector(false, 7, &mut lcd);
                            }
                            4 => {
                                let mut remove: bool = false;
                                for index in 0..4 {
                                    loop {
                                        if refresh {
                                            render_edit_screen(
                                                &preferences.format_watering_time(),
                                                index < 2,
                                                &mut lcd,
                                            );
                                            refresh = false;
                                        }

                                        delay.delay_ms(500);

                                        if update_date {
                                            preferences.tick_time();
                                        }
                                        update_date = !update_date;

                                        if up_button.is_high().unwrap()
                                            && down_button.is_high().unwrap()
                                        {
                                            remove = true;
                                            break;
                                        }

                                        if up_button.is_high().unwrap() {
                                            if preferences.watering.is_none() {
                                                preferences.set_default_watering_time();
                                            } else {
                                                match index {
                                                    0 => {
                                                        preferences.watering.unwrap().1 =
                                                            (preferences.watering.unwrap().1 + 1)
                                                                % 24;
                                                    }
                                                    1 => {
                                                        preferences.watering.unwrap().0 =
                                                            (preferences.watering.unwrap().0 + 1)
                                                                % 60;
                                                    }
                                                    2 => {
                                                        preferences.watering.unwrap().3 =
                                                            (preferences.watering.unwrap().3 + 1)
                                                                % 24;
                                                    }
                                                    3 => {
                                                        preferences.watering.unwrap().2 =
                                                            (preferences.watering.unwrap().2 + 1)
                                                                % 60;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            refresh = true;
                                        } else if down_button.is_high().unwrap() {
                                            if preferences.watering.is_none() {
                                                preferences.set_default_watering_time();
                                            } else {
                                                match index {
                                                    0 => {
                                                        preferences.watering.unwrap().1 =
                                                            (preferences.watering.unwrap().1 + 23)
                                                                % 24;
                                                    }
                                                    1 => {
                                                        preferences.watering.unwrap().0 =
                                                            (preferences.watering.unwrap().0 + 59)
                                                                % 60;
                                                    }
                                                    2 => {
                                                        preferences.watering.unwrap().3 =
                                                            (preferences.watering.unwrap().3 + 23)
                                                                % 24;
                                                    }
                                                    3 => {
                                                        preferences.watering.unwrap().2 =
                                                            (preferences.watering.unwrap().2 + 59)
                                                                % 60;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            refresh = true;
                                        } else if select_button.is_high().unwrap() {
                                            refresh = true;
                                            break;
                                        }
                                    }
                                    if remove {
                                        break;
                                    }
                                }
                                // Check legality
                                if !remove
                                    && ((preferences.watering.unwrap().1 > preferences.watering.unwrap().3) || // Hours are incorrect
                                        (preferences.watering.unwrap().1 == preferences.watering.unwrap().3 && // Minutes are incorrect assuming hours are equal
                                            preferences.watering.unwrap().0 > preferences.watering.unwrap().2))
                                {
                                    preferences.watering = Some((
                                        preferences.watering.unwrap().2,
                                        preferences.watering.unwrap().3,
                                        preferences.watering.unwrap().0,
                                        preferences.watering.unwrap().1,
                                    ));
                                }
                            }
                            _ => {
                                // Pressure has no configuration
                            }
                        }
                    }
                }
                _ => {
                    if smoke_detector.is_high().unwrap() {
                        // Panic!!!
                        let roof_open = &roof_vent.is_set_high().unwrap();
                        render_screen(FIRE, true, &mut lcd);
                        while smoke_detector.is_high().unwrap() {
                            // Enable sprinklers
                            sprinklers.set_high().unwrap();
                            // Ensure windows are closed
                            roof_vent.set_low().unwrap();
                            // Sound alarm
                            buzzer.set_high().unwrap();
                            delay.delay_ms(1000);
                            // Still keep track of time though
                            preferences.tick_time();
                        }
                        // Safe; Disable sprinklers and open vent if it was open before
                        buzzer.set_low().unwrap();
                        sprinklers.set_low().unwrap();
                        if *roof_open {
                            roof_vent.set_high().unwrap();
                        }
                    }

                    data = get_bme_data(&mut bme, &mut delay, &mut buzzer);

                    // Check if temperature is valid
                    let temp = get_temperature(&data);
                    if temp < preferences.temperature.0 || temp > preferences.temperature.1 {
                        // open vent
                        roof_vent.set_high().unwrap();
                    } else {
                        roof_vent.set_low().unwrap();
                    }

                    // Check if humidity is valid
                    let humidity = get_humidity(&data);
                    if humidity < preferences.humidity.0 || humidity > preferences.humidity.1 {
                        // enable sprinklers
                        sprinklers.set_high().unwrap();
                    } else {
                        sprinklers.set_low().unwrap();
                    }

                    // Check if it is watering time
                    if preferences.is_watering_time() {
                        sprinklers.set_high().unwrap();
                    } else {
                        sprinklers.set_low().unwrap();
                    }
                }
            }
        } else {
            continue;
        }

        let mut data_str: String<12> = String::new();
        match current_screen_index {
            0 => {
                // Temp
                uwrite!(&mut data_str, "Temp: {}F", get_temperature(&data)).unwrap(); // Str size 9
                render_screen(&data_str, true, &mut lcd);
                uwrite!(
                    &mut data_str,
                    "({}, {})",
                    preferences.temperature.0,
                    preferences.temperature.1
                )
                .unwrap(); // Str size 8
                render_screen(&data_str, false, &mut lcd);
            }
            1 => {
                // Humidity
                uwrite!(&mut data_str, "RH: {}%", get_humidity(&data)).unwrap(); // Str size 8
                render_screen(&data_str, true, &mut lcd);
                uwrite!(
                    &mut data_str,
                    "({}%, {}%)",
                    preferences.humidity.0,
                    preferences.humidity.1
                )
                .unwrap(); // Str size 12
                render_screen(&data_str, false, &mut lcd);
            }
            2 => {
                // Pressure
                uwrite!(&mut data_str, "PRS: {} mb", get_pressure(&data)).unwrap(); // Str size 12
                render_screen(&data_str, true, &mut lcd);
            }
            3 => {
                // Date
                let (time, date) = preferences.get_date_formatted();
                render_screen(&time, true, &mut lcd);
                render_screen(&date, false, &mut lcd);
            }
            _ => {
                // Water Schedule
                render_screen(&preferences.format_watering_time(), true, &mut lcd);
            }
        }
    }
}

#[derive(Format)]
enum RefreshAction {
    Up,
    Down,
    Select,
    Sensor,
}

/// Whether to update the LCD
/// param up: Up Button
/// param down: Down Button
/// param select: Selection Button
/// param wait_time: The amount of time between sensor polling
/// param preferences: Client Preferences
/// returns: if the LCD needs an update
fn should_update(
    up: &mut Pin<Gpio10, FunctionSio<SioInput>, PullDown>,
    down: &mut Pin<Gpio11, FunctionSio<SioInput>, PullDown>,
    select: &mut Pin<Gpio12, FunctionSio<SioInput>, PullDown>,
    wait_time: &mut u16,
    preferences: &mut Preferences,
) -> (bool, RefreshAction) {
    *wait_time += 1;
    // Make sure time is kept track of
    if *wait_time % 100 == 0 {
        preferences.tick_time();
    }

    // Prioritize button pressing
    if up.is_high().unwrap() {
        return (true, RefreshAction::Up);
    } else if down.is_high().unwrap() {
        return (true, RefreshAction::Down);
    } else if select.is_high().unwrap() {
        return (true, RefreshAction::Select);
    }

    // Check if sensors need updated
    if *wait_time >= 100 {
        *wait_time = 0; // TODO See if this actually works
        return (true, RefreshAction::Sensor);
    }
    (false, RefreshAction::Sensor) // It's ok to return SENSOR since it gets ignored
}

/// Ticks the cooldown for buttons
/// param cooldown: The amount of cooldown left
/// returns the new value for cooldown
fn tick_buttons(cooldown: u8) -> u8 {
    cooldown.saturating_sub(1)
}

/// Iterates forwards or backwards through Screens
/// param current_screen: The current screen being displayed
/// param next: Whether to iterate forward; If false, iterate backwards
/// returns: The next Screen
fn next_screen(mut current_screen_index: u8, next: bool) -> u8 {
    if next {
        current_screen_index = (current_screen_index + 1) % 5;
    } else {
        current_screen_index = (current_screen_index + 5 - 1) % 5;
    }
    info!("Next Screen: {}", current_screen_index);
    current_screen_index
}

fn debug(buzzer: &mut Pin<Gpio6, FunctionSio<SioOutput>, PullDown>, mut delay: Timer) {
    buzzer.set_high().unwrap();
    delay.delay_ms(500);
    buzzer.set_low().unwrap();
    delay.delay_ms(500);
}
