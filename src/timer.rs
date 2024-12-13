use panic_probe as _;

/// Contains a value that is decremented every millisecond
///
/// - **target_ms**: The current milliseconds remaining
///
/// ## Example:
/// ```rust
/// use gem_rs::timer::CountDownTimer;
///
/// let mut countdown = CountDownTimer::new(0); // Creates a new CountDownTimer with 0 milliseconds to wait
/// countdown.set_time(1000); // Sets the timer to 1000ms
///
/// // .. Delay for some time ...
/// countdown.tick(); // Make sure to tick the CountDownTimer every 1ms
///
/// if countdown.is_finished() {
///     // The CountDownTimer has reached 0
/// }
/// ```
pub struct CountDownTimer {
    target_ms: u16,
}

/// The delay in milliseconds between changing screens
pub const SCREEN_BUTTON_DELAY: u16 = 500;
/// The delay in milliseconds between updating uptime
pub const TICK_TIME_DELAY: u16 = 1000;
/// The delay in milliseconds between querying sensors
pub const SENSOR_DELAY: u16 = 2000;

impl CountDownTimer {
    /// Creates a new instances of CountDownTimer
    ///
    /// - param target_ms: The amount of milliseconds to wait when the CountDownTimer is created
    ///
    /// returns a new instances of CountDownTimer
    pub fn new(target_ms: u16) -> CountDownTimer {
        Self { target_ms }
    }

    /// Updates the CountDownTimer
    ///
    /// **NOTE:** This function should be called every millisecond
    pub fn tick(&mut self) {
        if self.target_ms > 0 {
            self.target_ms -= 1;
        }
    }

    /// Sets the waiting time for the CountDownTimer
    ///
    /// - param ms: The amount of milliseconds to set
    pub fn set_time(&mut self, ms: u16) {
        self.target_ms = ms;
    }

    /// Checks if the CountDownTimer has hit 0
    ///
    /// returns true if the CountDownTimer is at 0
    pub fn is_finished(&self) -> bool {
        self.target_ms == 0
    }
}
