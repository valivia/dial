use esp_hal::{handler, ram};

use crate::modules::dial::DIAL_DATA_PIN;

#[handler]
#[ram]
pub fn handler() {
    esp_println::println!(
        "GPIO Interrupt with priority {}",
        esp_hal::xtensa_lx::interrupt::get_level()
    );

    if critical_section::with(|cs| {
        DIAL_DATA_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Button was the source of the interrupt");
    } else {
        esp_println::println!("Button was not the source of the interrupt");
    }

    critical_section::with(|cs| {
        DIAL_DATA_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
