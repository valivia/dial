#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use modules::buttons::button_task;
use modules::dial::dial_task;
use modules::indicator::{Indication, IndicatorAction, indicator_task, set_indication};
use modules::mqtt::mqtt_init;
use modules::state::state_task;
use modules::wifi::wifi_init;

use crate::modules::buttons::service::ButtonService;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

pub mod actions;
pub mod modules;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    // State
    spawner.spawn(state_task().unwrap());

    // Buttons
    spawner.spawn(
        button_task(
            ButtonService::new(
                peripherals.GPIO36,
                peripherals.GPIO37,
                peripherals.GPIO35,
                peripherals.GPIO34,
            )
            .await,
        )
        .unwrap(),
    );

    // Dial
    spawner.spawn(dial_task(spawner, peripherals.GPIO4.into(), peripherals.GPIO5.into()).unwrap());

    // Indicators
    spawner.spawn(indicator_task(peripherals.GPIO21.into(), peripherals.GPIO2.into()).unwrap());

    set_indication(IndicatorAction {
        left: Indication::SingleFire(Duration::from_secs(1)),
        right: Indication::SingleFire(Duration::from_secs(1)),
    });

    // USB
    spawner.spawn(
        modules::usb::usb_init(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19).unwrap(),
    );

    // Wifi
    let wifi_stack = wifi_init(spawner, peripherals.WIFI).await;

    // MQTT
    spawner.spawn(mqtt_init(wifi_stack).unwrap());
}
