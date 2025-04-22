use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::{
    peripherals::{RADIO_CLK, TIMG0, WIFI},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{
    init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};
use heapless::String;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

static WIFI_SSID: &str = env!("WIFI_SSID");
static WIFI_PSK: &str = env!("WIFI_PASS");

pub async fn wifi_init(
    spawner: Spawner,
    timer1: TIMG0,
    radio: RADIO_CLK,
    wifi: WIFI,
    mut rng: Rng,
) -> Stack<'static> {
    let timg0 = TimerGroup::new(timer1);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), radio).unwrap()
    );

    let (controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();

    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    return stack;
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            info!("Connecting to {} ({})", WIFI_SSID, WIFI_PSK);
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: String::try_from(WIFI_SSID).unwrap(),
                password: String::try_from(WIFI_PSK).unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");

            println!("Scan");
            let (result, _) = controller.scan_n_async::<10>().await.unwrap();

            for ap in result {
                println!("{:?}", ap);
            }
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
