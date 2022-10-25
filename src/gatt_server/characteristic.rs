use crate::{
    gatt_server::descriptor::Descriptor,
    leaky_box_raw,
    utilities::{AttributeControl, AttributePermissions, BleUuid, CharacteristicProperties},
};
use esp_idf_sys::{
    esp_attr_control_t, esp_attr_value_t, esp_ble_gatts_add_char, esp_ble_gatts_set_attr_value,
    esp_nofail,
};
use log::{info, warn, debug};
use std::{
    fmt::Formatter,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Characteristic {
    /// The name of the characteristic, for debugging purposes.
    name: Option<String>,
    /// The characteristic identifier.
    pub(crate) uuid: BleUuid,
    /// The function to be called when a write happens. This functions receives the written value in the first parameter, a `Vec<u8>`.
    pub(crate) write_callback: Option<fn(Vec<u8>)>,
    /// A list of descriptors for this characteristic.
    pub(crate) descriptors: Vec<Arc<RwLock<Descriptor>>>,
    /// The handle that the Bluetooth stack assigned to this characteristic.
    pub(crate) attribute_handle: Option<u16>,
    /// The handle of the containing service.
    service_handle: Option<u16>,
    /// The access permissions for this characteristic.
    permissions: AttributePermissions,
    /// The properties that are announced for this characteristic.
    pub(crate) properties: CharacteristicProperties,
    /// The way this characteristic is read.
    pub(crate) control: AttributeControl,
    /// A buffer for keeping in memory the actual value of this characteristic.
    pub(crate) internal_value: Vec<u8>,
    /// The maximum length of the characteristic value.
    max_value_length: u16,
    /// A copy of the `control` property, in the `esp_attr_control_t` type, passed directly to the Bluetooth stack.
    internal_control: esp_attr_control_t,
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
            internal_control: AttributeControl::AutomaticResponse(vec![0]).into(),
            max_value_length: 8,
        }
    }

    /// Adds a [`Descriptor`] to the [`Characteristic`].
    pub fn add_descriptor<D: Into<Arc<RwLock<Descriptor>>>>(&mut self, descriptor: D) -> &mut Self {
        self.descriptors.push(descriptor.into());
        self
    }

    /// Sets the maximum length for the content of this characteristic. The default value is 8 bytes.
    pub fn value_length(&mut self, length: u16) -> &mut Self {
        self.max_value_length = length;
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
                    attr_max_len: self.max_value_length,
                    attr_len: self.internal_value.len() as u16,
                    attr_value: self.internal_value.as_mut_slice().as_mut_ptr(),
                }),
                &mut self.internal_control,
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
                .write()
                .unwrap()
                .register_self(self.service_handle.expect(
                    "Cannot register a descriptor to a characteristic without a service handle.",
                ));
        });
    }

    /// Sets the read callback for this characteristic.
    /// The callback willbe called when a client reads the value of this characteristic.
    ///
    /// The callback must return a `Vec<u8>` containing the value to be put into the response to the read request.
    ///
    /// # Notes
    ///
    /// The callback will be called from the Bluetooth stack's context, so it must not block.
    pub fn on_read(&mut self, callback: fn() -> Vec<u8>) -> &mut Self {
        if !self.properties.read || !self.permissions.read_access {
            warn!(
                "Characteristic {} does not have read permissions. Ignoring read callback.",
                self
            );

            return self;
        }

        self.control = AttributeControl::ResponseByApp(callback);
        self.internal_control = self.control.clone().into();

        self
    }

    /// Sets the write callback for this characteristic.
    /// The callback will be called when a client writes to this characteristic.
    ///
    /// The callback receives a `Vec<u8>` with the written value.
    /// It is up to the library user to decode the data into a meaningful format.
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

    /// Creates a new "User description" descriptor for this characteristic
    /// that contains the name of the characteristic.
    pub fn show_name_as_descriptor(&mut self) -> &mut Self {
        if let Some(name) = self.name.clone() {
            self.add_descriptor(Arc::new(RwLock::new(Descriptor::user_description(name))));
        }

        if let BleUuid::Uuid16(_) = self.uuid {
            warn!("You're specifying a user description for a standard characteristic. This might be useless.");
        }

        self
    }

    /// Sets the value of this [`Characteristic`].
    ///
    /// # Panics
    ///
    /// Panics if the value is too long and the characteristic is already registered.
    ///
    /// # Notes
    ///
    /// Before starting the server, you can freely set the value of a characteristic.
    /// The maximum value length will be derived from the length of the initial value.
    /// If you plan to expose only one data type for all the lifetime of the characteristic,
    /// then you'll never need to use the [`Self.value_length`] method, because
    /// the maximum size will be automatically set to the length of the latest value
    /// set before starting the server.
    pub fn set_value<T: Into<Vec<u8>>>(&mut self, value: T) -> &mut Self {
        let value: Vec<u8> = value.into();

        // If the characteristi hasn't been registered yet...
        if self.service_handle == None {
            // ...we can still change the value's maximum length.
            self.max_value_length = value.len() as u16;
        } else if value.len() > self.max_value_length as usize {
            // ...otherwise we MUST check that the value is smaller than the maximum.
            panic!("Value is too long for this characteristic and it can't be changed after starting the server.");
        }

        self.internal_value = value;
        self.control = AttributeControl::AutomaticResponse(self.internal_value.clone());
        self.internal_control = self.control.clone().into();

        info!(
            "Trying to set value of {} to {:?}.",
            self, self.internal_value
        );

        if let Some(handle) = self.attribute_handle {
            unsafe {
                esp_nofail!(esp_ble_gatts_set_attr_value(
                    handle,
                    self.internal_value.len() as u16,
                    self.internal_value.as_slice().as_ptr()
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
