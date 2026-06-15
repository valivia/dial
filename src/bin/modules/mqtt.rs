use core::{
    num::NonZero,
    sync::atomic::{AtomicBool, Ordering},
};
use defmt::{debug, error, info, warn};
use embassy_futures::select::{Either3, select3};
use embassy_net::{IpEndpoint, Stack, tcp::TcpSocket};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use heapless::String;
use rust_mqtt::{
    Bytes,
    buffer::*,
    client::{
        Client,
        event::Event,
        options::{
            ConnectOptions, PublicationOptions, RetainHandling, SubscriptionOptions, TopicReference,
        },
    },
    config::{KeepAlive, SessionExpiryInterval},
    types::{MqttBinary, MqttString, TopicFilter, TopicName, VarByteInt},
};

use crate::modules::indicator::{set_indication, signals::ACTION_SENT};

const TAG: &str = "[MQTT]";

static SERVER_IP: &str = env!("SERVER_IP");
static MQTT_USERNAME: &str = env!("MQTT_USERNAME");
static MQTT_PASSWORD: &str = env!("MQTT_PASSWORD");
static MQTT_CLIENT_ID: &str = env!("MQTT_CLIENT_ID");
static MQTT_PORT: &str = env!("MQTT_PORT");
static RECONNECT_DELAY: Duration = Duration::from_secs(5);

type MqttClient<'c> = Client<'c, TcpSocket<'c>, BumpBuffer<'c>, 1, 1, 1, 1>;

pub static MQTT_SIGNAL: Signal<CriticalSectionRawMutex, (String<32>, String<32>)> = Signal::new();
pub static MQTTT_CONNECTION_ACTIVE: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn mqtt_init(stack: Stack<'static>) {
    let mut tcp_rx = [0u8; 4096];
    let mut tcp_tx = [0u8; 4096];
    let mut mqtt_storage = [0u8; 1024];

    let endpoint = IpEndpoint::new(SERVER_IP.parse().unwrap(), MQTT_PORT.parse().unwrap());

    loop {
        stack.wait_config_up().await;

        if let Err(error) =
            mqtt_connect_and_run(stack, &mut tcp_rx, &mut tcp_tx, &mut mqtt_storage, endpoint).await
        {
            MQTTT_CONNECTION_ACTIVE.store(false, Ordering::Relaxed);

            warn!(
                "{} Reconnecting in {}s ({:?})",
                TAG,
                RECONNECT_DELAY.as_secs(),
                error
            );

            Timer::after(RECONNECT_DELAY).await;
        }
    }
}

async fn mqtt_connect_and_run<'c>(
    stack: Stack<'static>,
    rx_buffer: &'c mut [u8],
    tx_buffer: &'c mut [u8],
    mqtt_storage: &'c mut [u8],
    endpoint: IpEndpoint,
) -> Result<(), &'static str> {
    let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);

    socket.set_timeout(Some(Duration::from_secs(60)));

    info!("{} TCP connecting...", TAG);

    socket
        .connect(endpoint)
        .await
        .map_err(|_| "tcp connect failed")?;

    info!("{} TCP connected", TAG);

    let mut mqtt_buffer = BumpBuffer::new(mqtt_storage);

    let mut client = Client::<_, _, 1, 1, 1, 1>::new(&mut mqtt_buffer);

    client
        .connect(
            socket,
            &ConnectOptions::new()
                .clean_start()
                .session_expiry_interval(SessionExpiryInterval::Seconds(0))
                .keep_alive(KeepAlive::Seconds(NonZero::new(60).unwrap()))
                .user_name(MqttString::try_from(MQTT_USERNAME).unwrap())
                .password(MqttBinary::try_from(MQTT_PASSWORD).unwrap()),
            Some(MqttString::try_from(MQTT_CLIENT_ID).unwrap()),
        )
        .await
        .map_err(|_| "mqtt connect failed")?;

    let mut sub_options = SubscriptionOptions::new()
        .retain_handling(RetainHandling::SendIfNotSubscribedBefore)
        .retain_as_published()
        .at_least_once();

    if client.server_config().subscription_identifiers_supported {
        sub_options.subscription_identifier = Some(VarByteInt::from(42u16));
    }

    let topic = MqttString::from_str("owlimatronic/event").unwrap();

    let filter = TopicFilter::new(topic.as_borrowed()).unwrap();

    client
        .subscribe(filter.as_borrowed(), sub_options)
        .await
        .map_err(|_| "subscribe failed")?;

    info!("{} MQTT connected", TAG);

    MQTTT_CONNECTION_ACTIVE.store(true, Ordering::Relaxed);

    mqtt_run(&mut client).await?;

    Ok(())
}

async fn mqtt_run(client: &mut MqttClient<'_>) -> Result<(), &'static str> {
    loop {
        match select3(
            MQTT_SIGNAL.wait(),
            client.poll(),
            Timer::after(Duration::from_secs(30)),
        )
        .await
        {
            Either3::First((topic, payload)) => {
                info!("{} signal: {} {}", TAG, topic, payload);

                set_indication(ACTION_SENT);

                let topic_string = MqttString::from_str_unchecked(&topic);
                let topic_name = TopicName::new(topic_string.as_borrowed()).unwrap();
                let options =
                    PublicationOptions::new(TopicReference::Name(topic_name)).at_least_once();

                if let Err(error) = client
                    .publish(&options, Bytes::from(payload.as_bytes()))
                    .await
                {
                    warn!("{:?}", error);
                }
            }
            Either3::Second(result) => match result {
                Ok(Event::Pingresp) => (),
                Ok(event) => {
                    debug!("{:?}", event)
                }
                Err(e) => {
                    error!("{} MQTT error: {:?}", TAG, e);
                    return Err("poll failed");
                }
            },
            Either3::Third(_) => {
                client.ping().await.ok();
            }
        }
    }
}
