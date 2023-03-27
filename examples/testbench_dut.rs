use std::sync::{Arc, RwLock};

use bluedroid::{
    gatt_server::{Characteristic, Profile, Service, GLOBAL_GATT_SERVER},
    utilities::{AttributePermissions, BleUuid, CharacteristicProperties},
};

use esp_idf_sys::{esp_get_free_heap_size, esp_get_free_internal_heap_size};
use log::info;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    info!("Starting testbench server.");

    let char_value_write: Arc<RwLock<Vec<u8>>> =
        Arc::new(RwLock::new("Initial value".as_bytes().to_vec()));
    let char_value_read = char_value_write.clone();

    // A static characteristic.
    let static_characteristic = Characteristic::new(BleUuid::from_uuid128_string(
        "AF679F91-7239-402A-813D-55B5367E4A29",
    ))
    .name("Static Characteristic")
    .permissions(AttributePermissions::new().read())
    .max_value_length(20)
    .properties(CharacteristicProperties::new().read())
    .show_name()
    .set_value("Hello, world!".as_bytes().to_vec())
    .build();

    // A writable characteristic.
    let writable_characteristic = Characteristic::new(BleUuid::from_uuid128_string(
        "22E32A0E-1D8D-4300-B0DF-F996E44E65D3",
    ))
    .name("Writable Characteristic")
    .permissions(AttributePermissions::new().read().write())
    .properties(
        CharacteristicProperties::new()
            .read()
            .write_without_response(),
    )
    .on_read(move |_param| {
        info!("Read from writable characteristic.");
        return char_value_read.read().unwrap().clone();
    })
    .on_write(move |value, _param| {
        info!("Wrote to writable characteristic: {:?}", value);
        *char_value_write.write().unwrap() = value;
    })
    .show_name()
    .build();

    // A characteristic that notifies every second.
    let notifying_characteristic = Characteristic::new(BleUuid::from_uuid128_string(
        "6482DF69-A273-4F69-BADC-18583BA9A523",
    ))
    .name("Notifying Characteristic")
    .permissions(AttributePermissions::new().read())
    .properties(CharacteristicProperties::new().read().notify())
    .max_value_length(20)
    .show_name()
    .set_value("Initial value.".as_bytes().to_vec())
    .build();

    // A characteristic that notifies every second.
    let indicating_characteristic = Characteristic::new(BleUuid::from_uuid128_string(
        "B0D2A14A-8205-4E07-9317-DC7D61951473",
    ))
    .name("Indicating Characteristic")
    .permissions(AttributePermissions::new().read())
    .properties(CharacteristicProperties::new().read().indicate())
    .max_value_length(20)
    .show_name()
    .set_value("Initial value.".as_bytes().to_vec())
    .build();

    let advertised_service = Service::new(BleUuid::from_uuid128_string(
        "46548881-E7D9-4DE1-BBB7-DB016F1C657D",
    ))
    .name("Advertised Service")
    .primary()
    .characteristic(&static_characteristic)
    .characteristic(&writable_characteristic)
    .build();

    let another_service = Service::new(BleUuid::from_uuid128_string(
        "2BC08F60-17EB-431B-BEE7-329518164CD1",
    ))
    .name("Another Service")
    .primary()
    .characteristic(&notifying_characteristic)
    .characteristic(&indicating_characteristic)
    .build();

    let profile = Profile::new(0x0001)
        .name("Default Profile")
        .service(&advertised_service)
        .service(&another_service)
        .build();

    GLOBAL_GATT_SERVER
        .lock()
        .unwrap()
        .profile(profile)
        .device_name("BLUEDROID-DUT")
        .appearance(bluedroid::utilities::Appearance::GenericUnknown)
        .advertise_service(&advertised_service)
        .start();

    std::thread::spawn(move || {
        let mut counter = 0;
        loop {
            counter += 1;
            notifying_characteristic
                .write()
                .unwrap()
                .set_value(format!("Counter: {counter}").as_bytes().to_vec());
            indicating_characteristic
                .write()
                .unwrap()
                .set_value(format!("Counter: {counter}").as_bytes().to_vec());
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_millis(500));

        unsafe {
            let x = esp_get_free_heap_size();
            let y = esp_get_free_internal_heap_size();
            info!("Free heap: {x} bytes, free internal heap: {y} bytes");
        }
    });

    loop {
        info!("Main loop.");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
