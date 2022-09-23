use esp_idf_sys::{esp_bt_uuid_t, esp_gatt_id_t, ESP_UUID_LEN_16, ESP_UUID_LEN_128, ESP_UUID_LEN_32};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BleUuid {
    Uuid16(u16),
    Uuid32(u32),
    Uuid128([u8; 16]),
}

impl BleUuid {
    pub fn from_uuid16(uuid: u16) -> Self {
        BleUuid::Uuid16(uuid)
    }

    pub fn from_uuid32(uuid: u32) -> Self {
        BleUuid::Uuid32(uuid)
    }

    pub fn from_uuid128(uuid: [u8; 16]) -> Self {
        BleUuid::Uuid128(uuid)
    }

    pub fn as_uuid128_array(&self) -> [u8; 16] {
        let base_ble_uuid = [
            0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        match self {
            BleUuid::Uuid16(uuid) => {
                let mut uuid128 = base_ble_uuid;

                let mut uuid_as_bytes: [u8; 2] = uuid.to_be_bytes();
                uuid_as_bytes.reverse();

                uuid128[12..=13].copy_from_slice(&uuid_as_bytes[..]);
                uuid128
            }
            BleUuid::Uuid32(uuid) => {
                let mut uuid128 = base_ble_uuid;

                let mut uuid_as_bytes: [u8; 4] = uuid.to_be_bytes();
                uuid_as_bytes.reverse();

                uuid128[12..=15].copy_from_slice(&uuid_as_bytes[..]);
                uuid128
            }
            BleUuid::Uuid128(uuid) => *uuid,
        }
    }
}

impl From<BleUuid> for esp_gatt_id_t {
    fn from(val: BleUuid) -> Self {
        esp_gatt_id_t {
            uuid: val.into(),
            inst_id: 0x00,
        }
    }
}

impl From<BleUuid> for esp_bt_uuid_t {
    fn from(val: BleUuid) -> Self {
        let mut result: esp_bt_uuid_t = esp_bt_uuid_t::default();

        match val {
            BleUuid::Uuid16(uuid) => {
                result.len = ESP_UUID_LEN_16 as u16;
                result.uuid.uuid16 = uuid;
            }
            BleUuid::Uuid32(uuid) => {
                result.len = ESP_UUID_LEN_32 as u16;
                result.uuid.uuid32 = uuid;
            }
            BleUuid::Uuid128(uuid) => {
                result.len = ESP_UUID_LEN_128 as u16;
                result.uuid.uuid128 = uuid;
            }
        }

        result
    }
}

impl From<esp_bt_uuid_t> for BleUuid {
    fn from(uuid: esp_bt_uuid_t) -> Self {
        unsafe {
            match uuid.len {
                2 => BleUuid::Uuid16(uuid.uuid.uuid16),
                4 => BleUuid::Uuid32(uuid.uuid.uuid32),
                16 => BleUuid::Uuid128(uuid.uuid.uuid128),
                _ => panic!("Invalid UUID length."),
            }
        }
    }
}

impl From<esp_gatt_id_t> for BleUuid {
    fn from(uuid: esp_gatt_id_t) -> Self {
        Self::from(uuid.uuid)
    }
}

impl std::fmt::Display for BleUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BleUuid::Uuid16(uuid) => write!(f, "0x{:04x}", uuid),
            BleUuid::Uuid32(uuid) => write!(f, "0x{:08x}", uuid),
            BleUuid::Uuid128(uuid) => {
                let mut uuid = *uuid;
                uuid.reverse();

                let mut uuid_str = String::new();

                for byte in uuid.iter() {
                    uuid_str.push_str(&format!("{:02x}", byte));
                }
                uuid_str.insert(8, '-');
                uuid_str.insert(13, '-');
                uuid_str.insert(18, '-');

                write!(f, "{}", uuid_str)
            }
        }
    }
}

impl std::fmt::Debug for BleUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
