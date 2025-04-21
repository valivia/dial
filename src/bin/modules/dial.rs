use core::cell::RefCell;

use critical_section::Mutex;
use defmt::info;
use esp_hal::gpio::{AnyPin, Event, Input, Pull};

pub static DIAL_DATA_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[embassy_executor::task]
pub async fn run(pins: DialServiceGpio) {
    let mut button_service = DialService::new(pins);

    loop {
        button_service.run_loop().await;
    }
}

pub struct DialServiceGpio {
    pub data: AnyPin,
    pub mode: AnyPin,
}

pub struct DialService<'a> {
    mode_pin: Input<'a>,
}

impl<'a> DialService<'a> {
    pub fn new(pins: DialServiceGpio) -> Self {
        let pin_cfg = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up);

        let mut data_pin = Input::new(pins.data, pin_cfg.clone());
        let mode_pin = Input::new(pins.mode, pin_cfg.clone());

        critical_section::with(|cs| {
            data_pin.listen(Event::RisingEdge);
            DIAL_DATA_PIN.borrow_ref_mut(cs).replace(data_pin)
        });

        info!("Dial service initialized");

        Self {
            // GPIO
            mode_pin,
        }
    }

    pub async fn run_loop(&mut self) {
        // Implement the dial logic here
    }
}
