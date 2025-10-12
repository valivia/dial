use usbd_hid::descriptor::KeyboardUsage;

pub mod mqtt;
pub mod usb;

pub enum Action {
    Mqtt(mqtt::Action),
    Usb(usb::Action),
}

pub struct Page {
    pub actions: [Action; 5],
}

pub static PAGES: [Page; 2] = [
    Page {
        actions: [
            Action::Mqtt(mqtt::Action::new("phone/button/btn0")),
            Action::Mqtt(mqtt::Action::new("phone/button/btn1")),
            Action::Mqtt(mqtt::Action::new("phone/button/btn2")),
            Action::Mqtt(mqtt::Action::new("phone/button/btn3")),
            Action::Mqtt(mqtt::Action {
                min: 0,
                max: 10,
                dial_required: false,
                topic: "phone/button/btn4",
            }),
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
];
