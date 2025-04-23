use core::sync::atomic::{AtomicU32, Ordering};

use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use esp_hal::gpio::{AnyPin, Event, Input, Pull};

use crate::modules::indicator::{Indication, IndicatorAction, CURRENT_INDICATION};

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
pub static LAST_DIAL_COUNT_TIME: AtomicU32 = AtomicU32::new(0);
static KEEP_COUNT_DURATION: Duration = Duration::from_secs(5);

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
                    Timer::after_millis(1).await;
                    if self.mode_pin.is_low() {
                        info!("Dial mode bounced");
                        continue;
                    }
                    break;
                }
            }
        }

        if count > 0 {
            LAST_DIAL_COUNT.store(Some(count));
            info!("Dial ended, count: {}", count);

            let now = Instant::now().as_secs() as u32;

            LAST_DIAL_COUNT_TIME.store(now, Ordering::SeqCst);
            CURRENT_INDICATION.signal(IndicatorAction {
                left: Indication::SingleFire(KEEP_COUNT_DURATION),
                right: Indication::None,
            });

            // reset dial count after 5 seconds without blocking this loop
            embassy_executor::Spawner::for_current_executor()
                .await
                .spawn(reset_dial_count_after_delay(now))
                .ok();
        } else {
            LAST_DIAL_COUNT.store(None);
            info!("Dial ended, count: None");
        }
    }
}

#[embassy_executor::task]
async fn reset_dial_count_after_delay(time: u32) {
    Timer::after(KEEP_COUNT_DURATION).await;

    if LAST_DIAL_COUNT_TIME.load(Ordering::SeqCst) != time {
        info!("Cancelled dial count reset due to new event");
        return;
    }

    LAST_DIAL_COUNT.store(None);
    info!("Dial count reset after timeout");
}
