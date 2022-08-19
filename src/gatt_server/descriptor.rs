use crate::{leaky_box_raw, utilities::ble_uuid::BleUuid};
use esp_idf_sys::{esp_ble_gatts_add_char_descr, esp_nofail};
use log::info;

#[derive(Debug, Clone)]
pub struct Descriptor {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    value: Vec<u8>,
}

impl Descriptor {
    pub fn new(name: &str, uuid: BleUuid) -> Descriptor {
        Descriptor {
            name: Some(String::from(name)),
            uuid,
            value: Vec::new(),
        }
    }

    pub(crate) fn register_self(&mut self, service_handle: u16) {
        info!(
            "Registering {} into service at handle {}.",
            self, service_handle
        );

        unsafe {
            esp_nofail!(esp_ble_gatts_add_char_descr(
                service_handle,
                leaky_box_raw!(self.uuid.into()),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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
