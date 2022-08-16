use bluedroid::{
    gatt_server::{GattServer, Application, Service},
    utilities::ble_uuid::BleUuid,
};
use esp_idf_svc;
use log::info;
use bluedroid::gatt_server::{Characteristic, Descriptor};

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    let main_application = Application::new("Main Application", 0x01)
        .add_service(
            Service::new("Service 1", BleUuid::from_uuid16(0x0001), true)
                .add_characteristic(
                    Characteristic::new("Characteristic 1", BleUuid::from_uuid16(0x0001))
                        .add_descriptor(
                            &mut Descriptor::new("Descriptor 1", BleUuid::from_uuid16(0x0001))
                        )
                )
        );

    let applications = [main_application];

    let mut s = GattServer::take().unwrap();
    s.add_applications(&applications);
    s.start();
}
