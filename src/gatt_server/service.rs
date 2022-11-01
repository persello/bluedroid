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

/// Represents a GATT service.
#[derive(Debug, Clone)]
pub struct Service {
    name: Option<String>,
    pub(crate) uuid: BleUuid,
    pub(crate) characteristics: Vec<Arc<RwLock<Characteristic>>>,
    primary: bool,
    pub(crate) handle: Option<u16>,
}

impl Service {
    /// Creates a new [`Service`].
    #[must_use]
    pub const fn new(uuid: BleUuid) -> Self {
        Self {
            name: None,
            uuid,
            characteristics: Vec::new(),
            primary: false,
            handle: None,
        }
    }

    /// Sets the name of the [`Service`].
    ///
    /// This name is only used for debugging purposes.
    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the [`Service`] as primary.
    ///
    /// If you want your service to show up after an interrogation, you need to set it as primary.
    pub fn primary(&mut self) -> &mut Self {
        self.primary = true;
        self
    }

    /// Adds a [`Characteristic`] to the [`Service`].
    pub fn characteristic(&mut self, characteristic: &Arc<RwLock<Characteristic>>) -> &mut Self {
        self.characteristics.push(characteristic.clone());
        self
    }

    /// Returns a reference to the built [`Service`] behind an `Arc` and an `RwLock`.
    ///
    /// The returned value can be passed to any function of this crate that expects a [`Service`].
    /// It can be used in different threads, because it is protected by an `RwLock`.
    #[must_use]
    pub fn build(&self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self.clone()))
    }

    pub(crate) fn get_characteristic_by_handle(
        &self,
        handle: u16,
    ) -> Option<Arc<RwLock<Characteristic>>> {
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
                64, // TODO: count the number of characteristics and descriptors.
            ));
        }
    }

    pub(crate) fn register_characteristics(&mut self) {
        debug!("Registering {}'s characteristics.", &self);

        // Attention: The characteristics should be registered one after another.
        // We need to wait for the previous characteristic to be registered before we can register the next one.

        if self.characteristics.is_empty() {
            return;
        }

        // Loghi docet.

        let service_handle = self.handle.unwrap();
        let characteristics = self.characteristics.iter().cloned().zip(0..);
        let current_index = Arc::new(RwLock::new(0));

        for (characteristic, index) in characteristics {
            let i = current_index.clone();
            std::thread::spawn(move || {
                while *i.read().unwrap() != index {
                    std::thread::yield_now();
                }

                characteristic.write().unwrap().register_self(service_handle);

                while characteristic.read().unwrap().attribute_handle.is_none() {
                    std::thread::yield_now();
                }

                *i.write().unwrap() += 1;
            });
        }
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
