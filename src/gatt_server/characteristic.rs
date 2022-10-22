use crate::{
    gatt_server::descriptor::Descriptor,
    leaky_box_raw,
    utilities::{AttributeControl, AttributePermissions, BleUuid, CharacteristicProperties},
};
use esp_idf_sys::{
    esp_attr_value_t, esp_ble_gatts_add_char, esp_ble_gatts_set_attr_value, esp_nofail,
};
use log::{debug, warn};
use std::{cell::RefCell, fmt::Formatter, sync::{Arc, Mutex}};

#[derive(Debug, Clone)]
pub struct Characteristic {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    pub(crate) internal_value: Vec<u8>,
    pub(crate) write_callback: Option<fn(Vec<u8>)>,
    pub(crate) descriptors: Vec<Arc<Mutex<Descriptor>>>,
    pub(crate) attribute_handle: Option<u16>,
    service_handle: Option<u16>,
    permissions: AttributePermissions,
    pub(crate) properties: CharacteristicProperties,
    pub(crate) control: AttributeControl,
}

impl Characteristic {
    /// Creates a new [`Characteristic`].
    pub fn new(
        name: &str,
        uuid: BleUuid,
        permissions: AttributePermissions,
        properties: CharacteristicProperties,
    ) -> Self {
        Self {
            name: Some(String::from(name)),
            uuid,
            internal_value: vec![0],
            write_callback: None,
            descriptors: Vec::new(),
            attribute_handle: None,
            service_handle: None,
            permissions,
            properties,
            control: AttributeControl::AutomaticResponse(vec![0]),
        }
    }

    /// Adds a [`Descriptor`] to the [`Characteristic`].
    pub fn add_descriptor<D: Into<Arc<Mutex<Descriptor>>>>(
        &mut self,
        descriptor: D,
    ) -> &mut Self {
        self.descriptors.push(descriptor.into());
        self
    }

    pub(crate) fn get_descriptor(&self, handle: u16) -> Option<Arc<Mutex<Descriptor>>> {
        for descriptor in &self.descriptors {
            if descriptor.lock().unwrap().attribute_handle == Some(handle) {
                return Some(descriptor.clone());
            }
        }
        None
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
        self.descriptors.iter_mut().for_each(|descriptor| {
            descriptor
                .lock().unwrap()
                .register_self(self.service_handle.expect(
                    "Cannot register a descriptor to a characteristic without a service handle.",
                ));
        });
    }

    pub fn on_read(&mut self, response_type: AttributeControl) -> &mut Self {
        if !self.properties.read || !self.permissions.read_access {
            warn!(
                "Characteristic {} does not have read permissions. Ignoring read callback.",
                self
            );

            return self;
        }

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
        }

        // Else the callback is already set in the control property.

        self
    }

    pub fn on_write(&mut self, callback: fn(Vec<u8>)) -> &mut Self {
        if !((self.properties.write || self.properties.write_without_response)
            && self.permissions.write_access)
        {
            warn!(
                "Characteristic {} does not have write permissions. Ignoring write callback.",
                self
            );

            return self;
        }

        self.write_callback = Some(callback);
        self
    }

    pub fn show_name_as_descriptor(&mut self) -> &mut Self {
        if let Some(name) = self.name.clone() {
            self.add_descriptor(Arc::new(Mutex::new(Descriptor::user_description(name))));
        }

        if let BleUuid::Uuid16(_) = self.uuid {
            warn!("You're specifying a user description for a standard characteristic. This might be useless.");
        }

        self
    }

    pub fn set_value<T: Into<Vec<u8>>>(&mut self, value: T) -> &mut Self {
        self.internal_value = value.into();

        if let Some(handle) = self.attribute_handle {
            unsafe {
                esp_nofail!(esp_ble_gatts_set_attr_value(
                    handle,
                    self.internal_value.len() as u16,
                    self.internal_value.as_mut_slice().as_mut_ptr()
                ));
            }
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
