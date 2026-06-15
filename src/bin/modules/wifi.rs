use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{
    Config, ControllerConfig, DisconnectedStationInfo, Interface, WifiController, scan::ScanConfig,
    sta::StationConfig,
};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

static WIFI_SSID: &str = env!("WIFI_SSID");
static WIFI_PASSWORD: &str = env!("WIFI_PASS");
const TAG: &str = "[WIFI]";

pub async fn wifi_init(spawner: Spawner, wifi: WIFI<'static>) -> Stack<'static> {
    let station_config = Config::Station(
        StationConfig::default()
            .with_ssid(WIFI_SSID)
            .with_password(WIFI_PASSWORD.into()),
    );

    info!("{} Starting", TAG);
    let (mut controller, interfaces) = esp_radio::wifi::new(
        wifi,
        ControllerConfig::default().with_initial_config(station_config),
    )
    .unwrap();

    info!("{} Configured and started!", TAG);

    let wifi_interface = interfaces.station;
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    info!("{} Scanning", TAG);
    let scan_config = ScanConfig::default().with_max(10);
    let result = controller.scan_async(&scan_config).await.unwrap();
    for ap in result {
        info!("- {} ({})", ap.ssid.as_str(), ap.signal_strength);
    }

    spawner.spawn(connection(controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        info!("{} Got IP: {}", TAG, config.address);
    }

    return stack;
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("{} Start connection task", TAG);

    loop {
        info!("{} About to connect...", TAG);

        match controller.connect_async().await {
            Ok(info) => {
                info!("{} Connected to {:?}", TAG, info.ssid.as_str());

                fn get_ssid(info: &Option<DisconnectedStationInfo>) -> &str {
                    info.as_ref().map(|info| info.ssid.as_str()).unwrap_or("??")
                }

                // wait until we're no longer connected
                let info = controller.wait_for_disconnect_async().await.ok();
                warn!("{} Disconnected: {:?}", TAG, get_ssid(&info));
            }
            Err(e) => {
                error!("{} Failed to connect: {:?}", TAG, e);
            }
        }

        Timer::after(Duration::from_millis(5000)).await
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
