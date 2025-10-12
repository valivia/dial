use core::{
    net::Ipv4Addr,
    sync::atomic::{AtomicBool, Ordering},
};

use defmt::{error, info};
use embassy_net::{tcp::TcpSocket, Stack};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use heapless::String;
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    packet::v5::{publish_packet::QualityOfService as QoS, reason_codes::ReasonCode},
    utils::rng_generator::CountingRng,
};

use crate::modules::indicator::{set_indication, signals::ACTION_SENT};

const TAG: &str = "[MQTT]";

pub static MQTT_SIGNAL: Signal<CriticalSectionRawMutex, (String<32>, String<32>)> = Signal::new();
pub static MQTTT_CONNECTION_ACTIVE: AtomicBool = AtomicBool::new(false);

fn handle_mqtt_error(e: ReasonCode) -> bool {
    match e {
        ReasonCode::NetworkError => {
            MQTTT_CONNECTION_ACTIVE.store(false, Ordering::Relaxed);
            error!("{} Network Error", TAG);
            true
        }
        _ => {
            error!("{} Other Error: {:?}", TAG, e);
            false
        }
    }
}

#[embassy_executor::task]
pub async fn mqtt_init(stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut reconnect_delay_secs = 1;
    let mut first_run = true;

    loop {
        if !stack.is_config_up() {
            stack.wait_config_up().await;
            info!("{} WIFI stack is up, connecting to MQTT broker", TAG);
        }

        if first_run {
            first_run = false;
        } else {
            Timer::after(Duration::from_secs(reconnect_delay_secs)).await;
        }

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Addr::new(192, 168, 1, 50), 1883);
        info!("{} connecting...", TAG);

        if let Err(_) = socket.connect(remote_endpoint).await {
            error!("{} connect error", TAG);
            reconnect_delay_secs = (reconnect_delay_secs * 2).min(60);
            continue;
        }

        reconnect_delay_secs = 1;
        info!("{} TCP connected!", TAG);

        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );

        config.add_max_subscribe_qos(QoS::QoS1);
        config.add_client_id("dial");
        config.keep_alive = 120;
        config.max_packet_size = 100;

        let mut recv_buffer = [0; 256];
        let mut write_buffer = [0; 256];

        let mut client = MqttClient::<_, 5, _>::new(
            socket,
            &mut write_buffer,
            256,
            &mut recv_buffer,
            256,
            config,
        );

        info!("{} client created", TAG);

        if let Err(mqtt_error) = client.connect_to_broker().await {
            handle_mqtt_error(mqtt_error);
            continue;
        }

        MQTTT_CONNECTION_ACTIVE.store(true, Ordering::Relaxed);
        info!("{} connected to broker!", TAG);

        let mut last_activity = Instant::now();

        loop {
            // Wait for mqtt message to be queued or a timeout
            let result = with_timeout(Duration::from_millis(200), MQTT_SIGNAL.wait()).await;

            match result {
                // Message received
                Ok((topic, payload)) => {
                    info!("{} signal: {} {}", TAG, topic, payload);

                    set_indication(ACTION_SENT);

                    if let Err(e) = client
                        .send_message(topic.as_str(), payload.as_bytes(), QoS::QoS0, false)
                        .await
                    {
                        if handle_mqtt_error(e) {
                            break;
                        } else {
                            continue;
                        }
                    }

                    last_activity = Instant::now();
                }

                // Timeout occurred
                Err(_) => {
                    // check if we need to send a ping
                    if last_activity.elapsed() >= Duration::from_secs(8) {
                        if let Err(e) = client.send_ping().await {
                            if handle_mqtt_error(e) {
                                break;
                            } else {
                                continue;
                            }
                        }

                        last_activity = Instant::now();
                    }
                }
            }

            Timer::after(Duration::from_millis(100)).await;
        }

        error!("{} connection lost, retrying...", TAG);
    }
}
