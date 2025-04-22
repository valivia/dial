use embassy_time::Timer;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull};

use crate::modules::buttons::{ButtonState, BUTTON_SIGNAL};

use super::Button;

pub struct ButtonService<'a> {
    select_pin_1: Output<'a>,
    select_pin_2: Output<'a>,
    select_pin_3: Output<'a>,
    data_pin: Input<'a>,

    channel: usize,

    last_buttons_state: [bool; 8],
}

pub struct ButtonServiceGpio {
    pub select_1: AnyPin,
    pub select_2: AnyPin,
    pub select_3: AnyPin,
    pub data: AnyPin,
}

impl<'a> ButtonService<'a> {
    pub async fn new(pins: ButtonServiceGpio) -> Self {
        let select_pin_1 = Output::new(pins.select_1, Level::High, OutputConfig::default());
        let select_pin_2 = Output::new(pins.select_2, Level::High, OutputConfig::default());
        let select_pin_3 = Output::new(pins.select_3, Level::High, OutputConfig::default());

        let data_pin = Input::new(pins.data, InputConfig::default().with_pull(Pull::Up));

        let mut button_service = ButtonService {
            select_pin_1,
            select_pin_2,
            select_pin_3,
            data_pin,

            channel: 0,
            last_buttons_state: [false; 8],
        };

        button_service.set_channel(button_service.channel);
        Timer::after_millis(1).await;

        button_service
    }

    pub async fn run_loop(&mut self) {
        self.set_channel(self.channel);
        Timer::after_millis(1).await;

        let is_pressed = self.data_pin.is_low();

        if self.last_buttons_state[self.channel] != is_pressed {
            // Signal the button state change
            BUTTON_SIGNAL.signal((
                Button::from_index(self.channel),
                ButtonState::from(is_pressed),
            ));

            // Update the last state
            self.last_buttons_state[self.channel] = is_pressed;
        }

        self.channel = (self.channel + 1) % 8;
    }

    fn set_channel(&mut self, channel: usize) {
        let level = |bit: usize| {
            if (channel & (1 << bit)) != 0 {
                Level::High
            } else {
                Level::Low
            }
        };

        self.select_pin_1.set_level(level(0));
        self.select_pin_2.set_level(level(1));
        self.select_pin_3.set_level(level(2));
    }
}
