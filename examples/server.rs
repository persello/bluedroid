use std::sync::{Arc, RwLock, Mutex};

use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER};
use bluedroid::utilities::AttributeControl;
use bluedroid::{
    gatt_server::{Profile, Service},
    utilities::{AttributePermissions, BleUuid, CharacteristicProperties},
};
use embedded_hal::delay::blocking::DelayUs;
use lazy_static::lazy_static;
use log::info;

lazy_static! {
    // Keep track of a counter value.
    static ref COUNTER: std::sync::Mutex<u8> = std::sync::Mutex::new(0);
    // Keep track of a writable value.
    static ref WRITABLE: std::sync::Mutex<u8> = std::sync::Mutex::new(0);
}

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    let manufacturer_name_characteristic = Arc::new(RwLock::new(
        Characteristic::new(
            "Manufacturer Name",
            BleUuid::from_uuid16(0x2A29),
            AttributePermissions::read(),
            CharacteristicProperties::new().read(),
        )
        .on_read(AttributeControl::AutomaticResponse(
            "ESP32".as_bytes().to_vec(),
        ))
        .to_owned(),
    ));

    let model_number_characteristic = Arc::new(RwLock::new(
        Characteristic::new(
            "Model Number",
            BleUuid::from_uuid16(0x2A24),
            AttributePermissions::read(),
            CharacteristicProperties::new().read(),
        )
        .on_read(AttributeControl::AutomaticResponse(
            "ESP32".as_bytes().to_vec(),
        ))
        .to_owned(),
    ));

    let serial_number_characteristic = Arc::new(RwLock::new(Characteristic::new(
        "Serial Number",
        BleUuid::from_uuid16(0x2A25),
        AttributePermissions::read(),
        CharacteristicProperties::new().read(),
    )));

    let device_information_service = Arc::new(RwLock::new(
        Service::new("Device Information", BleUuid::from_uuid16(0x180A), true)
            .add_characteristic(manufacturer_name_characteristic)
            .add_characteristic(model_number_characteristic)
            .add_characteristic(serial_number_characteristic)
            .to_owned(),
    ));

    let main_profile = Profile::new("Main Profile", 0xAA).add_service(device_information_service);

    let heart_rate_characteristic = Arc::new(RwLock::new(
        Characteristic::new(
            "Heart Rate Measurement",
            BleUuid::from_uuid16(0x2A37),
            AttributePermissions::read(),
            CharacteristicProperties::new().read(),
        )
        .on_read(AttributeControl::ResponseByApp(|| {
            info!("Heart Rate Measurement callback called.");
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            format!("Heart rate, response #{}!", counter)
                .as_bytes()
                .to_vec()
        }))
        .show_name_as_descriptor()
        .to_owned(),
    ));

    let heart_rate_service = Arc::new(RwLock::new(
        Service::new("Heart Rate", BleUuid::from_uuid16(0x180D), true)
            .add_characteristic(heart_rate_characteristic.clone())
            .to_owned(),
    ));

    let secondary_profile = Profile::new("Secondary Profile", 0xBB).add_service(heart_rate_service);

    let custom_characteristic = Arc::new(RwLock::new(
        Characteristic::new(
            "Custom Characteristic",
            BleUuid::from_uuid128_string("FBFBFBFB-FBFB-FBFB-FBFB-FBFBFBFBFBFB"),
            AttributePermissions::read_write(),
            CharacteristicProperties::new()
                .read()
                .write_without_response(),
        )
        .on_read(AttributeControl::ResponseByApp(|| {
            info!("Custom Characteristic read callback called.");
            let writable = WRITABLE.lock().unwrap();
            format!("Custom Characteristic read, value is {}!", writable)
                .as_bytes()
                .to_vec()
        }))
        .on_write(|data| {
            info!("Custom Characteristic write callback called.");
            let mut writable = WRITABLE.lock().unwrap();
            *writable = data[0];
            info!("Custom Characteristic write, value is now {}!", writable);
        })
        .show_name_as_descriptor()
        .to_owned(),
    ));

    let custom_service = Arc::new(RwLock::new(
        Service::new(
            "Custom Service",
            BleUuid::from_uuid128_string("FAFAFAFA-FAFA-FAFA-FAFA-FAFAFAFAFAFA"), // FAR BETTER, RUN RUN RUN RUN RUN RUN RUN AWAY...
            true,
        )
        .add_characteristic(custom_characteristic)
        .to_owned(),
    ));

    let custom_profile = Profile::new("Custom Profile", 0xCC).add_service(custom_service);

    let profiles = [
        Arc::new(RwLock::new(main_profile)),
        Arc::new(RwLock::new(secondary_profile)),
        Arc::new(RwLock::new(custom_profile)),
    ];

    GLOBAL_GATT_SERVER
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .add_profiles(&profiles)
        .device_name("ESP32-GATT-Server")
        .appearance(bluedroid::utilities::Appearance::WristWornPulseOximeter)
        .advertise_service(Service::new(
            "Custom Service",
            BleUuid::from_uuid128_string("FAFAFAFA-FAFA-FAFA-FAFA-FAFAFAFAFAFA"),
            true,
        ))
        .start();

    let mut delay = esp_idf_hal::delay::FreeRtos;
    let mut val: u32 = 0;

    loop {
        delay.delay_ms(1000).unwrap();
        heart_rate_characteristic
            .write()
            .unwrap()
            .set_value(val.to_le_bytes());
        val += 1;
    }
}
