use std::fmt::Formatter;
use crate::gatt_server::descriptor::Descriptor;
use crate::utilities::ble_uuid::BleUuid;

#[derive(Debug, Clone)]
pub struct Characteristic {
    name: Option<String>,
    uuid: BleUuid,
    value: Vec<u8>,
    descriptors: Vec<Descriptor>,
}

impl Characteristic {
    pub fn new(name: &str, uuid: BleUuid) -> Characteristic {
        Characteristic {
            name: Some(String::from(name)),
            uuid,
            value: Vec::new(),
            descriptors: Vec::new(),
        }
    }

    pub fn add_descriptor(&mut self, descriptor: &mut Descriptor) -> &mut Self {
        self.descriptors.push(descriptor.clone());
        self
    }

    fn register_self(&mut self) {}

    fn register_descriptors(&self) {}
}

impl std::fmt::Display for Characteristic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.clone().unwrap_or_else(|| "Unnamed characteristic".to_string()), self.uuid)
    }
}
