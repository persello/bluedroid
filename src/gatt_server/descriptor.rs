use crate::utilities::ble_uuid::BleUuid;

#[derive(Debug, Clone)]
pub struct Descriptor {
    name: Option<String>,
    uuid: BleUuid,
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
}

impl std::fmt::Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.clone().unwrap_or_else(|| "Unnamed descriptor".to_string()), self.uuid)
    }
}