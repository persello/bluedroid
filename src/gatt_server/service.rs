use crate::{
    gatt_server::characteristic::Characteristic, gatt_server::descriptor::Descriptor,
    leaky_box_raw, utilities::BleUuid,
};
use esp_idf_sys::*;
use log::debug;
use std::{
    fmt::Formatter,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Service {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    pub(crate) characteristics: Vec<Arc<RwLock<Characteristic>>>,
    primary: bool,
    pub(crate) handle: Option<u16>,
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

    pub fn add_characteristic(&mut self, characteristic: Arc<RwLock<Characteristic>>) -> &mut Self {
        self.characteristics.push(characteristic);
        self
    }

    pub(crate) fn get_characteristic(&self, handle: u16) -> Option<Arc<RwLock<Characteristic>>> {
        self.characteristics
            .iter()
            .find(|characteristic| characteristic.read().unwrap().attribute_handle == Some(handle))
            .cloned()
    }

    pub(crate) fn get_characteristic_by_id(
        &self,
        id: esp_bt_uuid_t,
    ) -> Option<Arc<RwLock<Characteristic>>> {
        self.characteristics
            .iter()
            .find(|characteristic| characteristic.read().unwrap().uuid == id.into())
            .cloned()
    }

    pub(crate) fn get_descriptor_by_id(
        &self,
        id: esp_bt_uuid_t,
    ) -> Option<Arc<RwLock<Descriptor>>> {
        for characteristic in &self.characteristics {
            #[allow(clippy::significant_drop_in_scrutinee)]
            for descriptor in characteristic.read().unwrap().clone().descriptors {
                if descriptor.read().unwrap().uuid == id.into() {
                    return Some(descriptor);
                }
            }
        }

        None
    }

    pub(crate) fn register_self(&mut self, interface: u8) {
        debug!("Registering {} on interface {}.", &self, interface);

        let id: esp_gatt_srvc_id_t = esp_gatt_srvc_id_t {
            id: self.uuid.into(),
            is_primary: self.primary,
        };

        unsafe {
            esp_nofail!(esp_ble_gatts_create_service(
                interface,
                leaky_box_raw!(id),
                16, // TODO: count the number of characteristics and descriptors.
            ));
        }
    }

    pub(crate) fn register_characteristics(&mut self) {
        debug!("Registering {}'s characteristics.", &self);
        self.characteristics.iter().for_each(|characteristic| {
            characteristic.write().unwrap().register_self(
                self.handle
                    .expect("Cannot register a characteristic to a service without a handle."),
            );
        });
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed service".to_string()),
            self.uuid,
        )
    }
}
