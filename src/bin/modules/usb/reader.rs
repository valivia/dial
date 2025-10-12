use defmt::info;
use embassy_usb::{
    class::hid::{HidReader, ReportId, RequestHandler},
    control::OutResponse,
};
use esp_hal::otg_fs::asynch::Driver;

#[embassy_executor::task]
pub async fn usb_reader_task(reader: HidReader<'static, Driver<'static>, 1>) {
    let mut request_handler = MyRequestHandler {};

    reader.run(false, &mut request_handler).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}
