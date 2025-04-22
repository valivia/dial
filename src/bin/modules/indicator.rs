use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};

pub static INDICATOR_SERVICE: Mutex<RefCell<Option<IndicatorService>>> =
    Mutex::new(RefCell::new(None));

pub struct IndicatorServiceGpio {
    pub left: AnyPin,
    pub right: AnyPin,
}

pub struct IndicatorService<'a> {
    left_indicator: Output<'a>,
    right_indicator: Output<'a>,
}

impl<'a> IndicatorService<'a> {
    pub fn new(pins: IndicatorServiceGpio) -> Self {
        let config = OutputConfig::default();

        let left_indicator = Output::new(pins.left, Level::Low, config);
        let right_indicator = Output::new(pins.right, Level::Low, config);

        Self {
            left_indicator,
            right_indicator,
        }
    }

    pub fn set_left(&mut self, state: bool) {
        let level = if state { Level::High } else { Level::Low };
        self.left_indicator.set_level(level);
    }

    pub fn set_right(&mut self, state: bool) {
        let level = if state { Level::High } else { Level::Low };
        self.right_indicator.set_level(level);
    }
}
