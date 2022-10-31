use std::sync::{Arc, Mutex};

use crate::{
    gatt_server::Descriptor,
    utilities::{AttributePermissions, BleUuid},
};

use embedded_svc::storage::RawStorage;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::nvs_storage::EspNvsStorage;
use lazy_static::lazy_static;
use log::debug;

lazy_static! {
    static ref STORAGE: Mutex<EspNvsStorage> = Mutex::new(
        EspNvsStorage::new_default(
            Arc::new(
                EspDefaultNvs::new()
                    .expect("Cannot initialise the default NVS. Did you declare an NVS partition?")
            ),
            "ble",
            true
        )
        .expect("Cannot create a new NVS storage. Did you declare an NVS partition?")
    );
}

impl Descriptor {
    pub fn user_description<S: AsRef<str>>(description: S) -> Self {
        Self::new(
            "User Description",
            BleUuid::from_uuid16(0x2901),
            AttributePermissions::read(),
        )
        .set_value(description.as_ref().as_bytes().to_vec())
        .clone()
    }

    /// Creates a CCCD.
    ///
    /// The contents of the CCCD are stored in NVS and persisted across reboots.
    ///
    /// # Panics
    ///
    /// Panics if the NVS is not configured.
    #[must_use]
    pub fn cccd() -> Self {
        Self::new(
            "Client Characteristic Configuration",
            BleUuid::from_uuid16(0x2902),
            AttributePermissions::read_write(),
        )
        .on_read(|param| {
            let storage = STORAGE.lock().unwrap();

            // Get the descriptor handle.

            // TODO: Find the characteristic that contains the handle.
            // WARNING: Using the handle is incredibly stupid as the NVS is not erased across flashes.

            // Inject characteristic UUID into the callback. Fucking hell.
            // Option 1. Add parent references to every object in the tree.
            // Option 2. ??????????

            // Create a key from the connection address.
            let key = format!(
                "{:02X}{:02X}{:02X}{:02X}-{:04X}",
                /* param.bda[1], */ param.bda[2],
                param.bda[3],
                param.bda[4],
                param.bda[5],
                param.handle
            );

            // Prepare buffer and read correct CCCD value from non-volatile storage.
            let mut buf: [u8; 1] = [0; 1];
            storage
                .get_raw(&key, &mut buf)
                .expect("Cannot get raw value from the NVS. Did you declare an NVS partition?")
                .map_or_else(
                    || {
                        debug!("No CCCD value found for key {}.", key);
                        vec![0, 0]
                    },
                    |value| {
                        debug!("Read CCCD value: {:?} for key {}.", value, key);
                        value.0.to_vec()
                    },
                )
        })
        .on_write(|value, param| {
            let mut storage = STORAGE.lock().unwrap();

            // Create a key from the connection address.
            let key = format!(
                "{:02X}{:02X}{:02X}{:02X}-{:04X}",
                /* param.bda[1], */ param.bda[2],
                param.bda[3],
                param.bda[4],
                param.bda[5],
                param.handle
            );

            debug!("Write CCCD value: {:?} at key {}", value, key);

            // Write CCCD value to non-volatile storage.
            storage
                .put_raw(&key, &value)
                .expect("Cannot put raw value to the NVS. Did you declare an NVS partition?");
        })
        .clone()
    }
}
