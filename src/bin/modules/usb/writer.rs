use defmt::warn;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use embassy_usb::class::hid::HidWriter;
use esp_hal::otg_fs::asynch::Driver;
use usbd_hid::descriptor::KeyboardReport;

use crate::{
    actions::usb::{Action, TriggerType},
    modules::buttons::ButtonState,
};

pub static USB_ACTION: Signal<CriticalSectionRawMutex, (Action, ButtonState)> = Signal::new();

#[embassy_executor::task]
pub async fn usb_writer_task(writer: HidWriter<'static, Driver<'static>, 8>) {
    let mut usb_state = UsbState::new(writer);

    loop {
        let (action, state) = USB_ACTION.wait().await;
        usb_state.keyboard_action(action, state).await;
    }
}

struct UsbState {
    active_keys: [u8; 6],
    writer: HidWriter<'static, Driver<'static>, 8>,
}

impl UsbState {
    fn new(writer: HidWriter<'static, Driver<'static>, 8>) -> Self {
        UsbState {
            active_keys: [0; 6],
            writer,
        }
    }

    pub async fn keyboard_action(&mut self, action: Action, state: ButtonState) {
        match (action.trigger, state) {
            (TriggerType::Press, ButtonState::Pressed) => {
                self.add_key(action.keycode);
                self.report().await;

                Timer::after_millis(50).await;
                self.remove_key(action.keycode);
                self.report().await;

                return;
            }
            (TriggerType::Press, ButtonState::Released) => {}

            (TriggerType::Hold, ButtonState::Pressed) => self.add_key(action.keycode),
            (TriggerType::Hold, ButtonState::Released) => self.remove_key(action.keycode),

            (TriggerType::Toggle, ButtonState::Released) => return,
            (TriggerType::Toggle, ButtonState::Pressed) => self.toggle_key(action.keycode),
        }

        self.report().await;
    }

    fn add_key(&mut self, keycode: u8) {
        if self.active_keys.contains(&keycode) {
            return;
        }
        for slot in self.active_keys.iter_mut() {
            if *slot == 0 {
                *slot = keycode;
                break;
            }
        }
    }

    fn remove_key(&mut self, keycode: u8) {
        for slot in self.active_keys.iter_mut() {
            if *slot == keycode {
                *slot = 0;
                break;
            }
        }
    }

    fn toggle_key(&mut self, keycode: u8) {
        if self.active_keys.contains(&keycode) {
            self.remove_key(keycode);
        } else {
            self.add_key(keycode);
        }
    }

    async fn report(&mut self) {
        let report = KeyboardReport {
            keycodes: self.active_keys,
            leds: 0,
            modifier: 0,
            reserved: 0,
        };
        // Send the report.
        match self.writer.write_serialize(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };
    }
}
