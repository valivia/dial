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
            Action::Mqtt(mqtt::Action::new("phone/button/btn4")),
        ],
    },
    Page {
        actions: [
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: 0x68,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: 0x69,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: 0x6a,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: 0x6b,
            }),
            Action::Usb(usb::Action {
                trigger: usb::TriggerType::Toggle,
                keycode: 0x6c,
            }),
        ],
    },
];
