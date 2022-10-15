use esp_idf_sys::*;

#[derive(Clone, Copy, Debug)]
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

impl Default for CharacteristicProperties {
    fn default() -> Self {
        Self {
            broadcast: false,
            read: false,
            write_without_response: false,
            write: false,
            notify: false,
            indicate: false,
            authenticated_signed_writes: false,
            extended_properties: false,
        }
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
