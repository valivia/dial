use usbd_hid::descriptor::KeyboardUsage;

use crate::actions::mqtt::MapValuesOptions;

pub mod mqtt;
pub mod usb;

pub enum Action {
    None,
    Mqtt(mqtt::Action),
    Usb(usb::Action),
}

pub struct Page {
    pub actions: [Action; 5],
}

pub static PAGES: [Page; 3] = [
    Page {
        actions: [
            Action::Mqtt(mqtt::Action::new_lamp_control("phone/button/btn0")),
            Action::Mqtt(mqtt::Action::new_lamp_control("phone/button/btn1")),
            Action::Mqtt(mqtt::Action::new_lamp_control("phone/button/btn2")),
            Action::Mqtt(mqtt::Action::new_lamp_control("phone/button/btn3")),
            Action::Mqtt(mqtt::Action::new("phone/button/btn3")),
        ],
    },
    Page {
        actions: [
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Press,
                keycode: KeyboardUsage::KeyboardF13 as u8,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Press,
                keycode: KeyboardUsage::KeyboardF14 as u8,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Hold,
                keycode: KeyboardUsage::KeyboardF15 as u8,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: KeyboardUsage::KeyboardF15 as u8,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Hold,
                keycode: KeyboardUsage::KeyboardAa as u8,
            }),
        ],
    },
    Page {
        actions: [
            Action::Mqtt(mqtt::Action {
                topic: "owlimatronic/event",
                dial: mqtt::DialMode::MapValues(MapValuesOptions {
                    required: true,
                    values: &[
                        (1, "yap"),
                        (2, "test"),
                        (3, "hello"),
                        (4, "shocked"),
                        (5, "pick_up"),
                        (6, "panic"),
                        (7, "sweep"),
                    ],
                }),
            }),
            Action::None,
            Action::None,
            Action::None,
            Action::None,
        ],
    },
];
