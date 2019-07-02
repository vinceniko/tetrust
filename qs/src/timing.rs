pub const SECOND: f64 = 1000.0;
pub const UPDATES_PER_SEC: f64 = 16.0;
pub const MILLIS_PER_UPDATE: f64 = SECOND / UPDATES_PER_SEC;

pub static mut ELAPSED: f64 = MILLIS_PER_UPDATE;

pub fn set_elapsed(elapsed: f64) {
    unsafe {
        ELAPSED = elapsed;
    }
}

pub fn get_elapsed() -> f64 {
    unsafe {
        ELAPSED
    }
}

#[cfg(not(target_arch="wasm32"))]
use std::time::{Instant};

#[derive(Debug)]
pub struct Timer {
    last_update: f64,
    fall_update: f64,
    fall_rate: f64,

    #[cfg(not(target_arch="wasm32"))]
    pub test: Instant,
}

impl Timer {
    fn new(fall_rate: f64) -> Self {
        Timer {
            last_update: 0.0,
            fall_update: 0.0,
            fall_rate,

            #[cfg(not(target_arch="wasm32"))]
            test: Instant::now()
        }
    }

    pub fn update(&mut self) {
        set_elapsed(MILLIS_PER_UPDATE);
        self.fall_update += get_elapsed();
    }

    pub fn fall(&mut self) -> bool {
        if self.fall_update > self.fall_rate {
            self.fall_update = 0.0;

            return true
        }
        false
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(SECOND / 2.0)
    }
}