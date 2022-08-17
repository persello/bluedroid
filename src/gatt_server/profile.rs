use crate::gatt_server::service::Service;
use esp_idf_sys::*;
use log::info;

#[derive(Debug, Clone)]
pub struct Profile {
    name: Option<String>,
    services: Vec<Service>,
    identifier: u16,
    pub(crate) interface: Option<u8>,
    handle_counter: u16,
}

impl Profile {
    pub fn new(name: &str, identifier: u16) -> Self {
        Profile {
            name: Some(String::from(name)),
            services: Vec::new(),
            identifier,
            interface: None,
            handle_counter: 0,
        }
    }

    pub fn add_service(mut self, service: &Service) -> Self {
        self.services.push(service.clone());
        self
    }

    pub(crate) fn generate_handle(&mut self) -> u16 {
        self.handle_counter += 1;
        self.handle_counter
    }

    pub(crate) fn register_self(&self) {
        info!("Registering {}.", self);
        unsafe { esp_nofail!(esp_ble_gatts_app_register(self.identifier)) };
    }

    fn register_services(mut self) {
        info!("Registering {}'s services.", &self);
        let handle = self.generate_handle();
        self.services.iter_mut().for_each(|service| {
            service.register_self(self.interface.unwrap(), handle);
        });
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interface_string = if let Some(interface) = self.interface {
            format!("{}", interface)
        } else {
            String::from("None")
        };

        write!(
            f,
            "{} (0x{:02x}, interface: {})",
            self.name
                .clone()
                .unwrap_or_else(|| "Unnamed application".to_string()),
            self.identifier,
            interface_string
        )
    }
}
