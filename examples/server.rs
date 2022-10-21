use bluedroid::gatt_server::{Characteristic, Descriptor, GLOBAL_GATT_SERVER};
use bluedroid::utilities::AttributeControl;
use bluedroid::{
    gatt_server::{Profile, Service},
    utilities::{AttributePermissions, BleUuid, CharacteristicProperties},
};
use lazy_static::lazy_static;
use log::info;

// Keep track of a counter value.
lazy_static! {
    static ref COUNTER: std::sync::Mutex<u8> = std::sync::Mutex::new(0);
}

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    let main_profile = Profile::new("Main Profile", 0xAA).add_service(
        Service::new("Device Information", BleUuid::from_uuid16(0x180A), true)
            .add_characteristic(
                Characteristic::new(
                    "Manufacturer Name",
                    BleUuid::from_uuid16(0x2A29),
                    AttributePermissions::read(),
                    CharacteristicProperties::new().read(),
                )
                .response(AttributeControl::AutomaticResponse(
                    "pulse.loop".as_bytes().to_vec(),
                )),
            )
            .add_characteristic(
                Characteristic::new(
                    "Model Number",
                    BleUuid::from_uuid16(0x2A24),
                    AttributePermissions::read(),
                    CharacteristicProperties::new().read(),
                )
                .response(AttributeControl::AutomaticResponse(
                    "pulse.loop".as_bytes().to_vec(),
                )),
            )
            .add_characteristic(
                Characteristic::new(
                    "Serial Number",
                    BleUuid::from_uuid16(0x2A25),
                    AttributePermissions::read(),
                    CharacteristicProperties::new().read(),
                ),
            ),
    );

    let secondary_profile = Profile::new("Secondary Profile", 0xBB).add_service(
        Service::new("Heart Rate", BleUuid::from_uuid16(0x180D), true).add_characteristic(
            Characteristic::new(
                "Heart Rate Measurement",
                BleUuid::from_uuid16(0x2A37),
                AttributePermissions::read(),
                CharacteristicProperties::new().read(),
            )
            .response(AttributeControl::ResponseByApp(|| {
                info!("Heart Rate Measurement callback called.");
                let mut counter = COUNTER.lock().unwrap();
                *counter += 1;
                format!("Heart rate, response #{}!", counter)
                    .as_bytes()
                    .to_vec()
            }))
            .add_descriptor(
                Descriptor::new(
                    "Descriptor",
                    BleUuid::from_uuid16(0x2901),
                    AttributePermissions::read(),
                )
                .set_value("Heart Rate Measurement Descriptor".as_bytes().to_vec()),
            ),
        ),
    );

    let custom_profile = Profile::new("Custom Profile", 0xCC).add_service(
        Service::new(
            "Custom Service",
            BleUuid::from_uuid128_string("FAFAFAFA-FAFA-FAFA-FAFA-FAFAFAFAFAFA"), // FAR BETTER, RUN RUN RUN RUN RUN RUN RUN AWAY...
            true,
        )
        .add_characteristic(
            Characteristic::new(
                "Custom Characteristic",
                BleUuid::from_uuid128_string("FBFBFBFB-FBFB-FBFB-FBFB-FBFBFBFBFBFB"),
                AttributePermissions::read(),
                CharacteristicProperties::new().read(),
            )
            .add_descriptor(Descriptor::user_description(
                "This is a custom characteristic.",
            )),
        ),
    );

    let profiles = [main_profile, secondary_profile, custom_profile];

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
}
