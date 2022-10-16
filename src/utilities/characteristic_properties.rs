use esp_idf_sys::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct CharacteristicProperties {
    pub broadcast: bool,
    pub read: bool,
    pub write_without_response: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
    pub authenticated_signed_writes: bool,
    pub extended_properties: bool,
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
        self.notify = true;
        self
    }

    pub fn indicate(mut self) -> Self {
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
