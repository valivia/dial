#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use modules::buttons::button_task;
use modules::dial::dial_task;
use modules::indicator::{indicator_task, set_indication, Indication, IndicatorAction};
use modules::mqtt::mqtt_init;
use modules::state::state_task;
use modules::wifi::wifi_init;

use crate::modules::buttons::service::ButtonService;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

pub mod actions;
pub mod modules;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    let rng = Rng::new(peripherals.RNG);

    info!("Embassy initialized!");

    // State
    spawner.spawn(state_task()).unwrap();

    // Buttons
    spawner
        .spawn(button_task(
            ButtonService::new(
                peripherals.GPIO36,
                peripherals.GPIO37,
                peripherals.GPIO35,
                peripherals.GPIO34,
            )
            .await,
        ))
        .unwrap();

    // Dial
    spawner
        .spawn(dial_task(
            peripherals.GPIO4.into(),
            peripherals.GPIO5.into(),
        ))
        .unwrap();

    // Indicators
    spawner
        .spawn(indicator_task(
            peripherals.GPIO21.into(),
            peripherals.GPIO2.into(),
        ))
        .unwrap();

    set_indication(IndicatorAction {
        left: Indication::SingleFire(Duration::from_secs(1)),
        right: Indication::SingleFire(Duration::from_secs(1)),
    });

    // USB
    spawner
        .spawn(modules::usb::usb_init(
            peripherals.USB0,
            peripherals.GPIO20,
            peripherals.GPIO19,
        ))
        .unwrap();

    // Wifi
    let wifi_stack = wifi_init(spawner, peripherals.TIMG0, peripherals.WIFI, rng.clone()).await;

    // MQTT
    spawner.spawn(mqtt_init(wifi_stack)).ok();
}
