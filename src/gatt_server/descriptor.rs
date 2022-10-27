use crate::{
    leaky_box_raw,
    utilities::{AttributeControl, AttributePermissions, BleUuid},
};

use esp_idf_sys::{
    esp_attr_value_t, esp_ble_gatts_add_char_descr, esp_ble_gatts_set_attr_value, esp_nofail,
};
use log::{debug, info};

#[derive(Debug, Clone)]
pub struct Descriptor {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    value: Vec<u8>,
    pub(crate) attribute_handle: Option<u16>,
    // TODO: Private.
    pub permissions: AttributePermissions,
}

impl Descriptor {
    pub fn new(name: &str, uuid: BleUuid, permissions: AttributePermissions) -> Self {
        Self {
            name: Some(String::from(name)),
            uuid,
            value: Vec::new(),
            attribute_handle: None,
            permissions,
        }
    }

    // TODO: Finish.
    pub fn on_read() {}

    pub fn on_write() {}

    // TODO: Implement same mechanism as for characteristics.
    pub fn set_value<T: Into<Vec<u8>>>(&mut self, value: T) -> &mut Self {
        self.value = value.into();
        if let Some(handle) = self.attribute_handle {
            unsafe {
                esp_nofail!(esp_ble_gatts_set_attr_value(
                    handle,
                    self.value.len() as u16,
                    self.value.as_slice().as_ptr()
                ));
            }
        } else {
            info!(
                "Descriptor {} not registered yet, value will be set on registration.",
                self
            );
        }
        self
    }

    pub(crate) fn register_self(&mut self, service_handle: u16) {
        debug!(
            "Registering {} into service at handle 0x{:04x}.",
            self, service_handle
        );

        unsafe {
            esp_nofail!(esp_ble_gatts_add_char_descr(
                service_handle,
                leaky_box_raw!(self.uuid.into()),
                self.permissions.into(),
                leaky_box_raw!(esp_attr_value_t {
                    attr_max_len: self.value.len() as u16,
                    attr_len: self.value.len() as u16,
                    attr_value: self.value.as_mut_slice().as_mut_ptr(),
                }),
                // TODO: Add custom control.
                leaky_box_raw!(AttributeControl::AutomaticResponse(Vec::new()).into()),
            ));
        }
    }
}

impl std::fmt::Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed descriptor".to_string()),
            self.uuid
        )
    }
}
