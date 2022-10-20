use std::borrow::Borrow;

use crate::gatt_server::service::Service;
use esp_idf_sys::*;
use log::debug;

#[derive(Debug, Clone)]
pub struct Profile {
    name: Option<String>,
    pub(crate) services: Vec<Service>,
    pub(crate) identifier: u16,
    pub(crate) interface: Option<u8>,
}

impl Profile {
    pub fn new(name: &str, identifier: u16) -> Self {
        Profile {
            name: Some(String::from(name)),
            services: Vec::new(),
            identifier,
            interface: None,
        }
    }

    pub fn add_service<S: Borrow<Service>>(mut self, service: S) -> Self {
        self.services.push(service.borrow().to_owned());
        self
    }

    pub(crate) fn register_self(&self) {
        debug!("Registering {}.", self);
        unsafe { esp_nofail!(esp_ble_gatts_app_register(self.identifier)) };
    }

    pub(crate) fn register_services(&mut self) {
        debug!("Registering {}'s services.", &self);
        self.services.iter_mut().for_each(|service| {
            service.register_self(self.interface.unwrap());
        });
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (0x{:04x})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed profile".to_string()),
            self.identifier,
        )
    }
}
