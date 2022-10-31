use std::sync::{Arc, RwLock};

use crate::gatt_server::service::Service;
use esp_idf_sys::*;
use log::debug;

#[derive(Debug, Clone)]
pub struct Profile {
    name: Option<String>,
    pub(crate) services: Vec<Arc<RwLock<Service>>>,
    pub(crate) identifier: u16,
    pub(crate) interface: Option<u8>,
}

impl Profile {
    #[must_use]
    pub fn new(name: &str, identifier: u16) -> Self {
        Self {
            name: Some(String::from(name)),
            services: Vec::new(),
            identifier,
            interface: None,
        }
    }

    #[must_use]
    pub fn add_service<S: Into<Arc<RwLock<Service>>>>(mut self, service: S) -> Self {
        self.services.push(service.into());
        self
    }

    pub(crate) fn get_service(&self, handle: u16) -> Option<Arc<RwLock<Service>>> {
        for service in &self.services {
            if service.read().unwrap().handle == Some(handle) {
                return Some(service.clone());
            }
        }

        None
    }

    pub(crate) fn get_service_by_id(&self, id: esp_gatt_id_t) -> Option<Arc<RwLock<Service>>> {
        for service in &self.services {
            if service.read().unwrap().uuid == id.into() {
                return Some(service.clone());
            }
        }

        None
    }

    pub(crate) fn register_self(&self) {
        debug!("Registering {}.", self);
        unsafe { esp_nofail!(esp_ble_gatts_app_register(self.identifier)) };
    }

    pub(crate) fn register_services(&mut self) {
        debug!("Registering {}'s services.", &self);
        self.services.iter_mut().for_each(|service| {
            service
                .write()
                .unwrap()
                .register_self(self.interface.unwrap());
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
