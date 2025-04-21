use defmt::info;
use embassy_time::Timer;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull};

pub struct ButtonServiceGpio {
    pub select_1: AnyPin,
    pub select_2: AnyPin,
    pub select_3: AnyPin,
    pub data: AnyPin,
}

#[embassy_executor::task]
pub async fn run(pins: ButtonServiceGpio) {
    let mut button_service = ButtonService::new(pins);

    loop {
        button_service.run_loop().await;
    }
}

pub struct ButtonService<'a> {
    select_pin_1: Output<'a>,
    select_pin_2: Output<'a>,
    select_pin_3: Output<'a>,
    data_pin: Input<'a>,

    channel: u8,

    last_buttons_state: [bool; 8],
}

impl<'a> ButtonService<'a> {
    pub fn new(pins: ButtonServiceGpio) -> Self {
        let select_pin_1 = Output::new(pins.select_1, Level::High, OutputConfig::default());
        let select_pin_2 = Output::new(pins.select_2, Level::High, OutputConfig::default());
        let select_pin_3 = Output::new(pins.select_3, Level::High, OutputConfig::default());

        let data_pin = Input::new(pins.data, InputConfig::default().with_pull(Pull::Up));

        info!("Buttonservice initialized");

        Self {
            // GPIO
            select_pin_1,
            select_pin_2,
            select_pin_3,
            data_pin,

            // State
            channel: 0,
            last_buttons_state: [false; 8],
        }
    }

    pub async fn run_loop(&mut self) {
        self.set_channel(self.channel);
        Timer::after_millis(1).await;

        let button_state = self.data_pin.is_high();

        if self.channel == 5 || self.channel == 6 {
            let page_state = if self.channel == 5 {
                &mut self.last_buttons_state[5]
            } else {
                &mut self.last_buttons_state[6]
            };

            if button_state != *page_state {
                *page_state = button_state;
                if button_state == false {
                    let direction = if self.channel == 5 { -1 } else { 1 };

                    // TODO: send event to the main loop
                    info!(
                        "Page changed {}",
                        if direction == -1 { "left" } else { "right" }
                    );
                }
            }
        } else {
            if self.last_buttons_state[self.channel as usize] != button_state {
                // TODO: send event to the main loop
                info!(
                    "Button {} {}",
                    self.channel,
                    if button_state { "pressed" } else { "released" }
                );

                self.last_buttons_state[self.channel as usize] = button_state;
            }
        }

        self.channel = (self.channel + 1) % 8;
    }

    fn set_channel(&mut self, channel: u8) {
        let level = |bit: u8| {
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
