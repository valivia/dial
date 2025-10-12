pub mod reader;
pub mod writer;

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::info;
use embassy_executor::Spawner;
use embassy_usb::{
    class::hid::{HidReaderWriter, State},
    Builder, Handler,
};
use esp_hal::{
    otg_fs::asynch::Config,
    peripherals::{GPIO19, GPIO20},
};
use esp_hal::{
    otg_fs::{asynch::Driver, Usb},
    peripherals::USB0,
};

use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use crate::modules::usb::{reader::usb_reader_task, writer::usb_writer_task};

// Static buffers
static EP_OUT_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUFFER: StaticCell<[u8; 64]> = StaticCell::new();

static STATE: StaticCell<State> = StaticCell::new();
static DEVICE_HANDLER: StaticCell<MyDeviceHandler> = StaticCell::new();

#[embassy_executor::task]
pub async fn usb_init(usb0: USB0<'static>, dp_pin: GPIO20<'static>, dm_pin: GPIO19<'static>) {
    let usb = Usb::new(usb0, dp_pin, dm_pin);

    let ep_out_buffer = EP_OUT_BUFFER.init([0; 1024]);
    let config = Config::default();

    let driver = Driver::new(usb, ep_out_buffer, config);

    // Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Owl Corp");
    config.product = Some("Dial");
    config.serial_number = Some("1");

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 256]);
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 256]);
    let msos_descriptor = MSOS_DESCRIPTOR.init([0; 256]);
    let control_buf = CONTROL_BUFFER.init([0; 64]);

    let device_handler = DEVICE_HANDLER.init(MyDeviceHandler::new());

    let state = STATE.init(State::new());

    let mut builder = Builder::new(
        driver,
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    builder.handler(device_handler);

    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 64,
    };

    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, state, config);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let (reader, writer) = hid.split();

    let spawner = Spawner::for_current_executor().await;

    spawner.spawn(usb_writer_task(writer)).unwrap();
    spawner.spawn(usb_reader_task(reader)).unwrap();

    usb_fut.await;
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
