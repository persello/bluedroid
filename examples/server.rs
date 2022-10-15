use bluedroid::gatt_server::{Characteristic, Descriptor, GLOBAL_GATT_SERVER};
use bluedroid::{
    gatt_server::{GattServer, Profile, Service},
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
                AttributePermissions {
                    encryption_required: false,
                    read_access: true,
                    write_access: true,
                },
                CharacteristicProperties {
                    broadcast: false,
                    read: true,
                    write_without_response: false,
                    write: false,
                    notify: false,
                    indicate: false,
                    authenticated_signed_writes: false,
                    extended_properties: false,
                },
            )
            .add_descriptor(&mut Descriptor::new(
                "Descriptor",
                BleUuid::from_uuid16(0x2901),
                AttributePermissions {
                    read_access: true,
                    write_access: true,
                    encryption_required: false,
                },
            ).set_value("Manufacturer Name Descriptor".as_bytes().to_vec())),
        ),
    );

    let secondary_profile = Profile::new("Secondary Profile", 0xBB).add_service(
        Service::new("Heart Rate", BleUuid::from_uuid16(0x180D), true).add_characteristic(
            Characteristic::new(
                "Heart Rate Measurement",
                BleUuid::from_uuid16(0x2A37),
                AttributePermissions {
                    encryption_required: false,
                    read_access: true,
                    write_access: true,
                },
                CharacteristicProperties {
                    broadcast: false,
                    read: true,
                    write_without_response: false,
                    write: false,
                    notify: false,
                    indicate: false,
                    authenticated_signed_writes: false,
                    extended_properties: false,
                },
            )
            .add_descriptor(
                &mut Descriptor::new(
                    "Descriptor",
                    BleUuid::from_uuid16(0x2901),
                    AttributePermissions {
                        read_access: true,
                        write_access: true,
                        encryption_required: false,
                    },
                )
                .set_value("Heart Rate Measurement Descriptor".as_bytes().to_vec()),
            ),
        ),
    );

    let custom_profile = Profile::new("Custom Profile", 0xCC).add_service(
        Service::new(
            "Custom Service",
            BleUuid::from_uuid128([
                0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA, 0xFA,
                0xFA, 0xFA,
            ]),
            true,
        )
        .add_characteristic(
            Characteristic::new(
                "Custom Characteristic",
                BleUuid::from_uuid128([
                    0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB, 0xFB,
                    0xFB, 0xFB, 0xFB,
                ]),
                AttributePermissions {
                    encryption_required: false,
                    read_access: true,
                    write_access: true,
                },
                CharacteristicProperties {
                    broadcast: false,
                    read: true,
                    write_without_response: false,
                    write: false,
                    notify: false,
                    indicate: false,
                    authenticated_signed_writes: false,
                    extended_properties: false,
                },
            )
            .add_descriptor(
                Descriptor::new(
                    "Descriptor",
                    BleUuid::from_uuid16(0x2901),
                    AttributePermissions {
                        read_access: true,
                        write_access: true,
                        encryption_required: false,
                    },
                )
                .set_value("Custom Characteristic Descriptor".as_bytes().to_vec()),
            ),
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
