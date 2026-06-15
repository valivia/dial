use core::sync::atomic::{AtomicU32, Ordering};

use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_hal::gpio::{AnyPin, Event, Input, InputPin, Pull};

use crate::modules::indicator::{Indication, IndicatorAction, set_indication};

use super::util::OptionalAtomicU8;

const TAG: &str = "[DIAL]";

// Task
#[embassy_executor::task]
pub async fn dial_task(spawner: Spawner, mode: AnyPin<'static>, data: AnyPin<'static>) {
    let mut dial_service = DialService::new(mode, data);

    spawner.spawn(dial_timeout_task().unwrap());

    loop {
        dial_service.run_loop().await;
    }
}

// Service
static MIN_TICK_DURATION: Duration = Duration::from_millis(85);
pub static LAST_DIAL_COUNT: OptionalAtomicU8 = OptionalAtomicU8::new(None);
pub static LAST_DIAL_COUNT_TIME: AtomicU32 = AtomicU32::new(0);
static KEEP_COUNT_DURATION: Duration = Duration::from_secs(5);

pub static DIAL_END_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

pub struct DialService {
    mode_pin: Input<'static>,
    data_pin: Input<'static>,
}

impl DialService {
    pub fn new(mode: impl InputPin + 'static, data: impl InputPin + 'static) -> Self {
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
        info!("{} Started dialing", TAG);

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
                        debug!(
                            "{} Registered edge but too fast ({} ms)",
                            TAG,
                            elapsed.as_millis()
                        );
                        continue;
                    }

                    // Limit to 10
                    if count == 10 {
                        break;
                    }

                    count += 1;
                    last_count = Instant::now();
                }
                Either::Second(_) => {
                    Timer::after_millis(1).await;
                    if self.mode_pin.is_low() {
                        info!("{} Mode bounced", TAG);
                        continue;
                    }
                    break;
                }
            }
        }

        info!("{} Stopped dialing, count: {}", TAG, count);

        if count > 0 {
            LAST_DIAL_COUNT.store(Some(count));

            let now = Instant::now().as_secs() as u32;

            LAST_DIAL_COUNT_TIME.store(now, Ordering::SeqCst);
            set_indication(IndicatorAction {
                left: Indication::SingleFire(KEEP_COUNT_DURATION),
                right: Indication::None,
            });

            // reset dial count after X seconds
            DIAL_END_SIGNAL.signal(now);
        } else {
            LAST_DIAL_COUNT.store(None);
        }
    }
}

#[embassy_executor::task]
async fn dial_timeout_task() {
    loop {
        let time = DIAL_END_SIGNAL.wait().await;
        Timer::after(KEEP_COUNT_DURATION).await;

        if LAST_DIAL_COUNT_TIME.load(Ordering::SeqCst) != time {
            debug!("{} Cancelled dial count reset due to new event", TAG);
            return;
        }

        LAST_DIAL_COUNT.store(None);
        debug!("{} Count reset after timeout", TAG);
    }
}
