use embassy_time::Timer;
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};

use crate::modules::buttons::{ButtonState, BUTTON_SIGNAL};

use super::Button;

pub struct ButtonService {
    select_pin_1: Output<'static>,
    select_pin_2: Output<'static>,
    select_pin_3: Output<'static>,
    data_pin: Input<'static>,

    channel: usize,

    last_buttons_state: [bool; 8],
}

impl ButtonService {
    pub async fn new(
        select_1: impl OutputPin + 'static,
        select_2: impl OutputPin + 'static,
        select_3: impl OutputPin + 'static,
        data: impl InputPin + 'static,
    ) -> Self {
        let select_pin_1 = Output::new(select_1, Level::High, OutputConfig::default());
        let select_pin_2 = Output::new(select_2, Level::High, OutputConfig::default());
        let select_pin_3 = Output::new(select_3, Level::High, OutputConfig::default());

        let data_pin = Input::new(data, InputConfig::default().with_pull(Pull::Up));

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

        self.channel = (self.channel + 1) % 7;
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
