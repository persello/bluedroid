use bluedroid::gatt_server::{Characteristic, Descriptor, GLOBAL_GATT_SERVER};
use bluedroid::{
    gatt_server::{GattServer, Profile, Service},
    utilities::ble_uuid::BleUuid,
};
use log::info;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    let main_profile = Profile::new("Main profile", 0xAA).add_service(
        Service::new("Device Information", BleUuid::from_uuid16(0x180A), true).add_characteristic(
            Characteristic::new("Manufacturer Name", BleUuid::from_uuid16(0x2A29)).add_descriptor(
                &mut Descriptor::new("Descriptor 1", BleUuid::from_uuid16(0x5678)),
            ),
        ),
    );

    let secondary_profile = Profile::new("Secondary profile", 0xBB).add_service(
        Service::new("Secondary service", BleUuid::from_uuid16(0xBBBB), false).add_characteristic(
            Characteristic::new("Secondary characteristic", BleUuid::from_uuid16(0xCCCC)).add_descriptor(
                &mut Descriptor::new("Descriptor 1", BleUuid::from_uuid16(0x5678)),
            ),
        ),
    );

    let profiles = [main_profile, secondary_profile];

    GLOBAL_GATT_SERVER
    .lock()
    .unwrap()
    .as_mut()
    .unwrap()
    .add_profiles(&profiles)
    .start();
}
