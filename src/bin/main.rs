#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, gpio::Io};
use esp_println as _;
use modules::buttons::{self, ButtonServiceGpio};
use modules::dial::{DialService, DialServiceGpio};
use modules::interrupt::handler;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

pub mod modules;
pub mod actions;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    // Buttons
    spawner
        .spawn(buttons::run(ButtonServiceGpio {
            data: peripherals.GPIO34.into(),
            select_1: peripherals.GPIO36.into(),
            select_2: peripherals.GPIO37.into(),
            select_3: peripherals.GPIO35.into(),
        }))
        .unwrap();

    // Dial
    DialService::new(DialServiceGpio {
        data: peripherals.GPIO5.into(),
        mode: peripherals.GPIO4.into(),
    });

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
