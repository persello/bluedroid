use std::fmt::Formatter;
use crate::{gatt_server::characteristic::Characteristic, leaky_box_raw, utilities::ble_uuid::BleUuid};
use esp_idf_sys::*;
use log::info;

#[derive(Debug, Clone)]
pub struct Service {
    name: Option<String>,
    uuid: BleUuid,
    characteristics: Vec<Characteristic>,
    primary: bool,
    handle: Option<u16>,
}

impl Service {
    pub fn new(name: &str, uuid: BleUuid, primary: bool) -> Service {
        Service {
            name: Some(String::from(name)),
            uuid,
            characteristics: Vec::new(),
            primary,
            handle: None,
        }
    }

    pub fn add_characteristic(&mut self, characteristic: &mut Characteristic) -> &mut Self {
        self.characteristics.push(characteristic.clone());
        self
    }

    pub(crate) fn register_self(&mut self, interface: u8, handle: u16) {
        info!("Registering {} on interface {} at handle {:04x}.", &self, interface, handle);
        let id = esp_gatt_srvc_id_t {
            is_primary: true,
            id: self.uuid.into(),
        };
        self.handle = Some(handle);
        unsafe {
            esp_nofail!(esp_ble_gatts_create_service(interface, leaky_box_raw!(id), self.handle.unwrap()));
        }
    }

    // pub(crate) fn register_characteristics(&self) {
    //     info!("Registering {}'s characteristics.", self);
    //     self.characteristics.iter().for_each(|mut characteristic| {
    //         characteristic.register_self(self);
    //     });
    // }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handle_string = if let Some(handle) = self.handle {
            format!("0x{:04x}", handle)
        } else {
            String::from("None")
        };

        write!(f, "{} ({}, handle: {})", self.name.clone().unwrap_or_else(|| "Unnamed service".to_string()), self.uuid, handle_string)
    }
}
