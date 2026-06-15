use core::str::FromStr;
use core::{fmt::Write, sync::atomic::Ordering};
use defmt::{info, warn};
use embassy_time::Duration;
use heapless::String;

use crate::actions::mqtt::DialMode;
use crate::modules::indicator::signals::{ACTION_SENT, MQTT_CONNECTION_ERROR};
use crate::modules::usb::writer::USB_ACTION;
use crate::{
    actions::{Action, PAGES, mqtt, usb},
    modules::{
        indicator::{IndicatorAction, set_indication},
        mqtt::MQTTT_CONNECTION_ACTIVE,
    },
};

use super::{
    buttons::{BUTTON_SIGNAL, Button, ButtonState},
    dial::LAST_DIAL_COUNT,
    indicator::CANCEL_INDICATION,
    mqtt::MQTT_SIGNAL,
};

const TAG: &str = "[STATE]";

#[embassy_executor::task]
pub async fn state_task() {
    let mut state_manager = StateManager::new();
    info!("{} Initialized", TAG);

    loop {
        let (button, state) = BUTTON_SIGNAL.wait().await;
        state_manager.handle_signal(button, state);
    }
}

static PAGE_CHANGE_SIGNAL_DURATION: Duration = Duration::from_millis(200);

#[derive(Debug, defmt::Format)]
pub enum ActionFailReason {
    NotDefined,
    MissingValue,
    InvalidValue,
    ServiceInactive,
}

pub struct StateManager {
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
                warn!("{} Ignored non-page button: {:?}", TAG, button);
                return;
            }
        };

        info!("{} Page changed to {}", TAG, self.current_page_index);
    }

    fn handle_button_event(&mut self, button: Button, state: ButtonState) {
        let page = &PAGES[self.current_page_index];
        let action = &page.actions[button.to_index()];

        if let Err(error) = match action {
            Action::None => Err(ActionFailReason::NotDefined),
            Action::Mqtt(mqtt_action) => {
                if state != ButtonState::Pressed {
                    return;
                }
                self.run_mqtt_action(mqtt_action)
            }
            Action::Usb(usb_action) => Ok(self.run_usb_action(usb_action.clone(), state)),
        } {
            warn!("{} Action failed, {:?}", TAG, error)
        }
    }

    fn run_mqtt_action(&self, action: &mqtt::Action) -> Result<(), ActionFailReason> {
        let count = LAST_DIAL_COUNT.load();

        // Reset the dial count
        LAST_DIAL_COUNT.store(None);
        CANCEL_INDICATION.signal(());

        let payload: String<32> = match &action.dial {
            DialMode::None => String::from_str("").unwrap(),
            DialMode::Normal(options) => match count {
                Some(c) => {
                    let mut s: String<32> = String::new();
                    write!(s, "{}", c).ok();
                    s
                }
                None => {
                    if options.required {
                        return Err(ActionFailReason::MissingValue);
                    }
                    String::from_str("").unwrap()
                }
            },

            DialMode::MapRange(options) => match count {
                Some(c) => {
                    let mapped = options.map_value(c);
                    let mut s: String<32> = String::new();
                    write!(s, "{:.2}", mapped).ok();
                    s
                }
                None => {
                    if options.required {
                        return Err(ActionFailReason::MissingValue);
                    }
                    String::from_str("").unwrap()
                }
            },
            DialMode::MapValues(options) => match count {
                Some(c) => match options.values.iter().find(|entry| entry.0 == c as u32) {
                    Some(entry) => String::from_str(entry.1).unwrap(),
                    None => {
                        if options.required {
                            return Err(ActionFailReason::InvalidValue);
                        }
                        String::from_str("").unwrap()
                    }
                },
                None => {
                    if options.required {
                        return Err(ActionFailReason::MissingValue);
                    }
                    String::from_str("").unwrap()
                }
            },
        };

        if !MQTTT_CONNECTION_ACTIVE.load(Ordering::Relaxed) {
            set_indication(MQTT_CONNECTION_ERROR);
            return Err(ActionFailReason::ServiceInactive);
        }

        MQTT_SIGNAL.signal((String::<32>::from_str(action.topic).unwrap(), payload));

        Ok(())
    }

    fn run_usb_action(&self, action: usb::Action, state: ButtonState) {
        set_indication(ACTION_SENT);
        info!("{} Running USB action: {:?}", TAG, action.keycode);
        USB_ACTION.signal((action, state));
    }
}
