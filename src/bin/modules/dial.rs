use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_time::{with_timeout, Duration, Instant};
use esp_hal::gpio::{AnyPin, Event, Input, Pull};

use super::util::OptionalAtomicU8;

// Task
#[embassy_executor::task]
pub async fn dial_task(mode: AnyPin, data: AnyPin) {
    let mut dial_service = DialService::new(mode, data);

    loop {
        dial_service.run_loop().await;
    }
}

// Service

static MIN_TICK_DURATION: Duration = Duration::from_millis(85);
pub static LAST_DIAL_COUNT: OptionalAtomicU8 = OptionalAtomicU8::new(None);

pub struct DialService<'a> {
    mode_pin: Input<'a>,
    data_pin: Input<'a>,
}

impl<'a> DialService<'a> {
    pub fn new(mode: AnyPin, data: AnyPin) -> Self {
        let pin_cfg = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up);

        let mut data_pin = Input::new(data, pin_cfg.clone());
        let mut mode_pin = Input::new(mode, pin_cfg.clone());

        data_pin.listen(Event::RisingEdge);
        mode_pin.listen(Event::AnyEdge);

        Self {
            // GPIO
            mode_pin,
            data_pin,
        }
    }

    pub async fn run_loop(&mut self) {
        self.mode_pin.wait_for_falling_edge().await;
        let mut count: u8 = 0;
        let mut last_count = Instant::now();
        info!("dial start");

        loop {
            match select(
                self.data_pin.wait_for_rising_edge(),
                with_timeout(Duration::from_secs(5), self.mode_pin.wait_for_rising_edge()),
            )
            .await
            {
                Either::First(_) => {
                    let elapsed = last_count.elapsed();
                    if elapsed < MIN_TICK_DURATION {
                        info!("Registered edge but too fast ({} ms)", elapsed.as_millis());
                        continue;
                    }

                    if count == 10 {
                        break;
                    }

                    count += 1;
                    last_count = Instant::now();
                }
                Either::Second(_) => {
                    break;
                }
            }
        }


        if count > 0 {
            LAST_DIAL_COUNT.store(Some(count));
            info!("Dial ended, count: {}", count);
        } else {
            LAST_DIAL_COUNT.store(None);
            info!("Dial ended, count: None");
        }
    }
}
