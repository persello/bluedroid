use bluedroid::gatt_server::{Characteristic, Descriptor, GLOBAL_GATT_SERVER};
use bluedroid::utilities::AttributeControl;
use bluedroid::{
    gatt_server::{Profile, Service},
    utilities::{AttributePermissions, BleUuid, CharacteristicProperties},
};
use log::info;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Logger initialised.");

    let main_profile = Profile::new("Main Profile", 0xAA).add_service(
        Service::new("Device Information", BleUuid::from_uuid16(0x180A), true).add_characteristic(
            Characteristic::new(
                "Manufacturer Name",
                BleUuid::from_uuid16(0x2A29),
                AttributePermissions::read(),
                CharacteristicProperties::new().read(),
            )
            .response(AttributeControl::AutomaticResponse(
                "pulse.loop".as_bytes().to_vec(),
            ))
            .add_descriptor(
                Descriptor::new(
                    "Descriptor",
                    BleUuid::from_uuid16(0x2901),
                    AttributePermissions::read(),
                )
                .set_value("Manufacturer Name Descriptor".as_bytes().to_vec()),
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
                "Heart rate, response by app!".as_bytes().to_vec()
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
        .register_profiles(&profiles)
        .start();
}
