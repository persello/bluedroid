use crate::{
    gatt_server::descriptor::Descriptor,
    leaky_box_raw,
    utilities::{AttributeControl, AttributePermissions, BleUuid, CharacteristicProperties},
};
use esp_idf_sys::{
    esp_attr_value_t, esp_ble_gatts_add_char, esp_ble_gatts_set_attr_value, esp_nofail,
};
use log::debug;
use std::{borrow::Borrow, fmt::Formatter};

#[derive(Debug, Clone)]
pub struct Characteristic {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    internal_value: Vec<u8>,
    internal_callback: Option<fn() -> Vec<u8>>,
    pub(crate) descriptors: Vec<Descriptor>,
    pub(crate) attribute_handle: Option<u16>,
    service_handle: Option<u16>,
    permissions: AttributePermissions,
    properties: CharacteristicProperties,
    pub(crate) control: AttributeControl,
}

impl Characteristic {
    /// Creates a new [`Characteristic`].
    pub fn new(
        name: &str,
        uuid: BleUuid,
        permissions: AttributePermissions,
        properties: CharacteristicProperties,
    ) -> Characteristic {
        Characteristic {
            name: Some(String::from(name)),
            uuid,
            internal_value: vec![0],
            internal_callback: None,
            descriptors: Vec::new(),
            attribute_handle: None,
            service_handle: None,
            permissions,
            properties,
            control: AttributeControl::AutomaticResponse(vec![0]),
        }
    }

    /// Adds a [`Descriptor`] to the [`Characteristic`].
    pub fn add_descriptor<D: Borrow<Descriptor>>(&mut self, descriptor: D) -> &mut Self {
        self.descriptors.push(descriptor.borrow().to_owned());
        self
    }

    /// Registers the [`Characteristic`] at the given service handle.
    pub(crate) fn register_self(&mut self, service_handle: u16) {
        debug!(
            "Registering {} into service at handle 0x{:04x}.",
            self, service_handle
        );
        self.service_handle = Some(service_handle);

        if let AttributeControl::AutomaticResponse(_) = self.control && self.internal_value.is_empty() {
            panic!("Automatic response requires a value to be set.");
        }

        unsafe {
            esp_nofail!(esp_ble_gatts_add_char(
                service_handle,
                leaky_box_raw!(self.uuid.into()),
                self.permissions.into(),
                self.properties.into(),
                leaky_box_raw!(esp_attr_value_t {
                    attr_max_len: self.internal_value.len() as u16,
                    attr_len: self.internal_value.len() as u16,
                    attr_value: self.internal_value.as_mut_slice().as_mut_ptr(),
                }),
                leaky_box_raw!(self.control.clone().into()),
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
        debug!("Registering {}'s descriptors.", &self);
        self.descriptors
            .iter_mut()
            .for_each(|descriptor: &mut Descriptor| {
                descriptor.register_self(self.service_handle.expect(
                    "Cannot register a descriptor to a characteristic without a service handle.",
                ));
            });
    }

    pub fn response(&mut self, response_type: AttributeControl) -> &mut Self {
        self.control = response_type;

        // If the response type is an automatic response, we need to update the value.
        if let AttributeControl::AutomaticResponse(value) = &self.control {
            self.internal_value = value.clone();

            if let Some(handle) = self.attribute_handle {
                unsafe {
                    esp_nofail!(esp_ble_gatts_set_attr_value(
                        handle,
                        self.internal_value.len() as u16,
                        self.internal_value.as_mut_slice().as_mut_ptr()
                    ));
                }
            }
        } else if let AttributeControl::ResponseByApp(callback) = &self.control {
            self.internal_callback = Some(*callback);
        }

        self
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
