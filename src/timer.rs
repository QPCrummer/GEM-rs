use rp_pico::hal::timer::CountDown;

pub enum CountdownDelay {
    ScreenButtonDelay(CountDown),
    EditButtonDelay(CountDown),
    SensorDelay(CountDown),
}

pub fn set_countdown(delay: CountdownDelay, milliseconds: u16) {

}