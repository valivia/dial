use core::net::Ipv4Addr;

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

pub static MQTT_SIGNAL: Signal<CriticalSectionRawMutex, (String<32>, String<32>)> = Signal::new();

fn handle_mqtt_error(e: ReasonCode) -> bool {
    match e {
        ReasonCode::NetworkError => {
            error!("MQTT Network Error");
            true
        }
        _ => {
            error!("Other MQTT Error: {:?}", e);
            false
        }
    }
}

#[embassy_executor::task]
pub async fn mqtt_init(stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut reconnect_delay_secs = 1;

    loop {
        Timer::after(Duration::from_secs(reconnect_delay_secs)).await;

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Addr::new(192, 168, 1, 50), 1883);
        info!("mqtt connecting...");

        if let Err(_) = socket.connect(remote_endpoint).await {
            error!("connect error");
            reconnect_delay_secs = (reconnect_delay_secs * 2).min(60);
            continue;
        }

        reconnect_delay_secs = 1;
        info!("mqtt TCP connected!");

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

        info!("mqtt client created");

        if let Err(mqtt_error) = client.connect_to_broker().await {
            handle_mqtt_error(mqtt_error);
            continue;
        }

        info!("mqtt connected to broker!");

        let mut last_activity = Instant::now();

        loop {
            let result = with_timeout(Duration::from_millis(200), MQTT_SIGNAL.wait()).await;

            match result {
                Ok((topic, payload)) => {
                    info!("mqtt signal: {} {}", topic, payload);

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

                Err(_) => {
                    // Timeout expired: check if we need to send a ping
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

        error!("mqtt connection lost, retrying...");
    }
}
