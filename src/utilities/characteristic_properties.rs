use esp_idf_sys::*;
use log::warn;

#[derive(Clone, Copy, Debug, Default)]
pub struct CharacteristicProperties {
    broadcast: bool,
    pub(crate) read: bool,
    pub(crate) write_without_response: bool,
    pub(crate) write: bool,
    pub(crate) notify: bool,
    pub(crate) indicate: bool,
    authenticated_signed_writes: bool,
    extended_properties: bool,
}

impl CharacteristicProperties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub fn read(mut self) -> Self {
        self.read = true;
        self
    }

    pub fn write_without_response(mut self) -> Self {
        self.write_without_response = true;
        self
    }

    pub fn write(mut self) -> Self {
        self.write = true;
        self
    }

    pub fn notify(mut self) -> Self {
        if self.indicate {
            warn!("Cannot set notify and indicate at the same time. Ignoring notify.");
            return self;
        }

        self.notify = true;
        self
    }

    pub fn indicate(mut self) -> Self {
        if self.notify {
            warn!("Cannot set notify and indicate at the same time. Ignoring indicate.");
            return self;
        }

        self.indicate = true;
        self
    }

    pub fn authenticated_signed_writes(mut self) -> Self {
        self.authenticated_signed_writes = true;
        self
    }

    pub fn extended_properties(mut self) -> Self {
        self.extended_properties = true;
        self
    }
}

impl From<CharacteristicProperties> for esp_gatt_char_prop_t {
    fn from(properties: CharacteristicProperties) -> Self {
        let mut result = 0;
        if properties.broadcast {
            result |= ESP_GATT_CHAR_PROP_BIT_BROADCAST;
        }
        if properties.read {
            result |= ESP_GATT_CHAR_PROP_BIT_READ;
        }
        if properties.write_without_response {
            result |= ESP_GATT_CHAR_PROP_BIT_WRITE_NR;
        }
        if properties.write {
            result |= ESP_GATT_CHAR_PROP_BIT_WRITE;
        }
        if properties.notify {
            result |= ESP_GATT_CHAR_PROP_BIT_NOTIFY;
        }
        if properties.indicate {
            result |= ESP_GATT_CHAR_PROP_BIT_INDICATE;
        }
        if properties.authenticated_signed_writes {
            result |= ESP_GATT_CHAR_PROP_BIT_AUTH;
        }
        if properties.extended_properties {
            result |= ESP_GATT_CHAR_PROP_BIT_EXT_PROP;
        }
        result as Self
    }
}
