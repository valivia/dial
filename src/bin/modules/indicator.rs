use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};

pub struct LoopingIndication {
    pub time_on: Duration,
    pub time_off: Duration,
    pub count: u8,
}

pub enum Indication {
    SingleFire(Duration),
    Looping(LoopingIndication),
    None,
}

pub struct IndicatorAction {
    pub left: Indication,
    pub right: Indication,
}

static CURRENT_INDICATION: Signal<CriticalSectionRawMutex, IndicatorAction> = Signal::new();
pub static CANCEL_INDICATION: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn indicator_task(left: AnyPin, right: AnyPin) {
    let config = OutputConfig::default();

    let mut left_indicator = Output::new(left, Level::Low, config);
    let mut right_indicator = Output::new(right, Level::Low, config);

    loop {
        let action = CURRENT_INDICATION.wait().await;
        CANCEL_INDICATION.reset();

        // Spawn two concurrent tasks to handle left and right independently.
        embassy_futures::join::join(
            handle_indication(&mut left_indicator, &action.left),
            handle_indication(&mut right_indicator, &action.right),
        )
        .await;

        Timer::after(Duration::from_millis(500)).await;
    }
}

pub fn set_indication(action: IndicatorAction) {
    CANCEL_INDICATION.signal(());
    CURRENT_INDICATION.signal(action);
}

/// Handles a single pin's indication pattern.
async fn handle_indication(pin: &mut Output<'_>, indication: &Indication) {
    match indication {
        Indication::None => {
            pin.set_low();
        }
        Indication::SingleFire(duration) => {
            pin.set_high();
            let _ = select(Timer::after(*duration), CANCEL_INDICATION.wait()).await;
            pin.set_low();
        }
        Indication::Looping(looping) => {
            for _ in 0..looping.count {
                pin.set_high();
                if let Either::Second(_) =
                    select(Timer::after(looping.time_on), CANCEL_INDICATION.wait()).await
                {
                    pin.set_low();
                    return;
                }

                pin.set_low();
                if let Either::Second(_) =
                    select(Timer::after(looping.time_off), CANCEL_INDICATION.wait()).await
                {
                    return;
                }
            }
        }
    }
}
