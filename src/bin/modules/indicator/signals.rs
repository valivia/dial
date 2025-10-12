use embassy_time::Duration;

use crate::modules::indicator::{Indication, IndicatorAction, LoopingIndication};

pub const MQTT_CONNECTION_ERROR: IndicatorAction = IndicatorAction {
    left: Indication::None,
    right: Indication::Looping(LoopingIndication {
        time_on: Duration::from_millis(100),
        time_off: Duration::from_millis(500),
        count: 3,
    }),
};

pub const WIFI_CONNECTION_FAILED: IndicatorAction = IndicatorAction {
    left: Indication::None,
    right: Indication::Looping(LoopingIndication {
        time_on: Duration::from_millis(500),
        time_off: Duration::from_millis(100),
        count: 3,
    }),
};

pub const ACTION_SENT: IndicatorAction = IndicatorAction {
    left: Indication::None,
    right: Indication::SingleFire(Duration::from_millis(100)),
};
