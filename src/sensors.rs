use bme680::{Bme680, FieldData, FieldDataCondition, PowerMode};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use i2c_pio::I2C;
use rp_pico::hal::gpio::bank0::{Gpio6, Gpio8, Gpio9};
use rp_pico::hal::gpio::{FunctionNull, FunctionSio, Pin, PullDown, SioOutput};
use rp_pico::hal::pio::SM0;
use rp_pico::hal::Timer;
use rp_pico::pac::PIO0;

use panic_probe as _;

type Bme<'a> = Bme680<
    I2C<'a, PIO0, SM0, Pin<Gpio8, FunctionNull, PullDown>, Pin<Gpio9, FunctionNull, PullDown>>,
    Timer,
>;

/// Gets data from the BME sensor
/// param bme: BME sensor instance
/// param delayer: BME sensor delay
/// param alarm: Buzzer Pin
/// returns FieldData
pub fn get_bme_data(
    bme: &mut Bme,
    delayer: &mut Timer,
    alarm: &mut Pin<Gpio6, FunctionSio<SioOutput>, PullDown>,
) -> FieldData {
    prep_bme(bme, delayer, alarm);
    bme.get_sensor_data(delayer)
        .unwrap_or((FieldData::default(), FieldDataCondition::Unchanged))
        .0
}

/// Gets temperature in Fahrenheit
/// param data: FieldData from get_bme_data()
pub fn get_temperature(data: &FieldData) -> u8 {
    (data.temperature_celsius() * (9. / 5.) + 32.) as u8
}

/// Gets percent humidity (whole number)
/// param data: FieldData from get_bme_data()
pub fn get_humidity(data: &FieldData) -> u8 {
    data.humidity_percent() as u8
}

/// Gets atmospheric pressure in millibars
/// param data: FieldData from get_bme_data()
pub fn get_pressure(data: &FieldData) -> u16 {
    data.pressure_hpa() as u16
}

/// Sets the sensor's mode to Forced
/// This should be called before getting data
/// If there is an error setting up, an alarm is sounded
/// param bme: BME sensor reference
/// param delayer: BME delay
/// param alarm: Buzzer Pin
pub fn prep_bme(
    bme: &mut Bme,
    delayer: &mut Timer,
    alarm: &mut Pin<Gpio6, FunctionSio<SioOutput>, PullDown>,
) {
    if bme.set_sensor_mode(delayer, PowerMode::ForcedMode).is_err() {
        loop {
            alarm.set_high().unwrap();
            delayer.delay_ms(500);
            alarm.set_low().unwrap();
            delayer.delay_ms(1000);
        }
    }
}
