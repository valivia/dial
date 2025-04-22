use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use service::{ButtonService, ButtonServiceGpio};

pub mod service;

// Button
#[derive(Clone, Copy, PartialEq, defmt::Format)]
pub enum Button {
    PageLeft,
    PageRight,
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
}

impl Button {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Button::Button1,
            1 => Button::Button2,
            2 => Button::Button3,
            3 => Button::Button4,
            4 => Button::Button5,
            5 => Button::PageLeft,
            6 => Button::PageRight,
            _ => panic!("Invalid button index"),
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Button::Button1 => 0,
            Button::Button2 => 1,
            Button::Button3 => 2,
            Button::Button4 => 3,
            Button::Button5 => 4,
            Button::PageLeft => 5,
            Button::PageRight => 6,
        }
    }
}

// ButtonState
#[derive(Clone, Copy, PartialEq, defmt::Format)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<bool> for ButtonState {
    fn from(value: bool) -> Self {
        if value {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        }
    }
}

pub static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, (Button, ButtonState)> = Signal::new();

// Main task
#[embassy_executor::task]
pub async fn button_task(pins: ButtonServiceGpio) {
    let mut button_service = ButtonService::new(pins).await;
    info!("Button service initialized");

    loop {
        button_service.run_loop().await;
    }
}
