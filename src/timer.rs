use rp_pico::hal::fugit::ExtU32;
use rp_pico::hal::timer::CountDown;

use panic_probe as _;

pub enum CountdownDelay {
    ScreenButtonDelay,
    EditButtonDelay,
    SensorDelay,
}

/// Starts the countdown for the specified CountdownDelay
/// ScreenButtonDelay: 500ms
/// EditButtonDelay: 200ms
/// SensorDelay: 4s
/// param countdown_type: CountdownDelay instance
/// param countdown: CountDown instance
pub fn start_countdown(countdown_type: CountdownDelay, mut countdown: &CountDown) {
    match countdown_type {
        CountdownDelay::ScreenButtonDelay() => countdown.start(500.millis()),
        CountdownDelay::EditButtonDelay() => countdown.start(200.millis()),
        CountdownDelay::SensorDelay() => countdown.start(4.secs()),
    }
}

/// Detects if the specified CountdownDelay is finished
/// param countdown_type: CountdownDelay instance
/// /// param countdown: CountDown instance
/// returns if the CountdownDelay is not counting
pub fn countdown_ended(countdown_type: CountdownDelay, mut countdown: &CountDown) -> bool {
    match countdown_type {
        CountdownDelay::ScreenButtonDelay
        | CountdownDelay::EditButtonDelay
        | CountdownDelay::SensorDelay => is_finished(&mut countdown),
    }
}

fn is_finished(countdown: &mut CountDown) -> bool {
    match countdown.wait() {
        Err(nb::Error::WouldBlock) => {
            false
        }
        _ => {
            true
        }
    }
}