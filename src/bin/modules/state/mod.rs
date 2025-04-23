use core::fmt::Write;
use core::str::FromStr;
use defmt::{info, warn};
use embassy_time::Duration;
use heapless::String;

use crate::{
    actions::{mqtt, usb, Action, PAGES},
    modules::indicator::{set_indication, IndicatorAction},
};

use super::{
    buttons::{Button, ButtonState, BUTTON_SIGNAL},
    dial::LAST_DIAL_COUNT,
    indicator::CANCEL_INDICATION,
    mqtt::MQTT_SIGNAL,
};

#[embassy_executor::task]
pub async fn state_task() {
    let mut state_manager = StateManager::new();
    info!("State manager initialized");

    loop {
        let (button, state) = BUTTON_SIGNAL.wait().await;
        state_manager.handle_signal(button, state);
    }
}

static PAGE_CHANGE_SIGNAL_DURATION: Duration = Duration::from_millis(200);

pub struct StateManager {
    // Index of PAGES (static sized array)
    current_page_index: usize,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            current_page_index: 0,
        }
    }

    pub fn handle_signal(&mut self, button: Button, state: ButtonState) {
        match button {
            Button::PageRight | Button::PageLeft => self.handle_page_event(button, state),
            _ => self.handle_button_event(button, state),
        }
    }

    fn handle_page_event(&mut self, button: Button, state: ButtonState) {
        if state != ButtonState::Pressed {
            return;
        }

        let page_count = PAGES.len();
        self.current_page_index = match button {
            Button::PageRight => {
                set_indication(IndicatorAction::single_fire(
                    None,
                    Some(PAGE_CHANGE_SIGNAL_DURATION),
                ));
                (self.current_page_index + 1) % page_count
            }
            Button::PageLeft => {
                set_indication(IndicatorAction::single_fire(
                    Some(PAGE_CHANGE_SIGNAL_DURATION),
                    None,
                ));
                (self.current_page_index + page_count - 1) % page_count
            }
            _ => {
                warn!("Ignored non-page button: {:?}", button);
                return;
            }
        };

        info!("Page changed to {}", self.current_page_index);
    }

    fn handle_button_event(&mut self, button: Button, state: ButtonState) {
        let page = &PAGES[self.current_page_index];
        let action = &page.actions[button.to_index()];

        match action {
            Action::Mqtt(mqtt_action) => self.run_mqtt_action(mqtt_action, state),
            Action::Usb(usb_action) => self.run_usb_action(usb_action),
        }
    }

    fn run_mqtt_action(&self, action: &mqtt::Action, state: ButtonState) {
        if state != ButtonState::Pressed {
            return;
        }

        let payload = match LAST_DIAL_COUNT.load() {
            Some(count) => {
                let mut s = String::<32>::new();
                write!(s, "{}", action.map_value(count)).unwrap();
                s
            }
            None => String::from_str("").unwrap(),
        };

        // Reset the dial count
        LAST_DIAL_COUNT.store(None);
        CANCEL_INDICATION.signal(());

        MQTT_SIGNAL.signal((String::<32>::from_str(action.topic).unwrap(), payload));
    }

    fn run_usb_action(&self, action: &usb::Action) {
        info!("Running USB action: {:?}", action.keycode);
    }
}
