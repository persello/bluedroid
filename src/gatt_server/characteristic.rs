use crate::{gatt_server::descriptor::Descriptor, leaky_box_raw, utilities::ble_uuid::BleUuid};
use esp_idf_sys::{esp_ble_gatts_add_char, esp_nofail};
use log::info;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct Characteristic {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    value: Vec<u8>,
    pub(crate) descriptors: Vec<Descriptor>,
    service_handle: Option<u16>,
}

impl Characteristic {
    /// Creates a new [`Characteristic`].
    pub fn new(name: &str, uuid: BleUuid) -> Characteristic {
        Characteristic {
            name: Some(String::from(name)),
            uuid,
            value: Vec::new(),
            descriptors: Vec::new(),
            service_handle: None,
        }
    }

    /// Adds a [`Descriptor`] to the [`Characteristic`].
    pub fn add_descriptor(&mut self, descriptor: &mut Descriptor) -> &mut Self {
        self.descriptors.push(descriptor.clone());
        self
    }

    /// Registers the [`Characteristic`] at the given service handle.
    pub(crate) fn register_self(&mut self, service_handle: u16) {
        info!(
            "Registering {} into service at handle {}.",
            self, service_handle
        );
        self.service_handle = Some(service_handle);

        unsafe {
            esp_nofail!(esp_ble_gatts_add_char(
                service_handle,
                leaky_box_raw!(self.uuid.into()),
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ));
        }
    }

    /// Registers the descriptors of this [`Characteristic`].
    ///
    /// This function should be called on the event of the characteristic being registered.
    ///
    /// # Panics
    ///
    /// Panics if the service handle is not registered.
    ///
    /// # Notes
    ///
    /// Bluedroid does not offer a way to register descriptors to a specific characteristic.
    /// This is simply done by registering the characteristic and then registering its descriptors.
    pub(crate) fn register_descriptors(&mut self) {
        info!("Registering {}'s descriptors.", &self);
        self.descriptors
            .iter_mut()
            .for_each(|descriptor: &mut Descriptor| {
                descriptor.register_self(self.service_handle.expect(
                    "Cannot register a descriptor to a characteristic without a service handle.",
                ));
            });
    }
}

impl std::fmt::Display for Characteristic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed characteristic".to_string()),
            self.uuid
        )
    }
}
