use panic_probe as _;

pub struct CountDownTimer {
    target_ms: u16,
}

pub const SCREEN_BUTTON_DELAY: u16 = 500; // 500ms ideally
pub const TICK_TIME_DELAY: u16 = 1000;
pub const SENSOR_DELAY: u16 = 2000; // 2000ms ideally

impl CountDownTimer {
    pub fn new(target_ms: u16) -> CountDownTimer {
        Self {
            target_ms,
        }
    }

    pub fn tick(&mut self) {
        if self.target_ms > 0 {
            self.target_ms -= 1;
        }
    }

    pub fn set_time(&mut self, ms: u16) {
        self.target_ms = ms;
    }

    pub fn is_finished(&self) -> bool {
        self.target_ms == 0
    }
}