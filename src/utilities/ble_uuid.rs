use esp_idf_sys::esp_gatt_id_t;

#[derive(Copy, Clone)]
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

                let uuid_as_bytes: [u8; 2] = uuid.to_be_bytes();

                uuid128[12..13].copy_from_slice(&uuid_as_bytes[..]);
                uuid128
            }
            BleUuid::Uuid32(uuid) => {
                let mut uuid128 = base_ble_uuid;

                let uuid_as_bytes: [u8; 4] = uuid.to_be_bytes();

                uuid128[12..15].copy_from_slice(&uuid_as_bytes[..]);
                uuid128
            }
            BleUuid::Uuid128(uuid) => *uuid,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<esp_gatt_id_t> for BleUuid {
    fn into(self) -> esp_gatt_id_t {
        let mut result = esp_gatt_id_t::default();
        match self {
            BleUuid::Uuid16(uuid) => {
                result.uuid.uuid.uuid16 = uuid;
                result.uuid.len = 2;
            }
            BleUuid::Uuid32(uuid) => {
                result.uuid.uuid.uuid32 = uuid;
                result.uuid.len = 4;
            }
            BleUuid::Uuid128(uuid) => {
                result.uuid.uuid.uuid128 = uuid;
                result.uuid.len = 16;
            }
        }

        result
    }
}

impl std::fmt::Display for BleUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BleUuid::Uuid16(uuid) => write!(f, "0x{:02x}", uuid),
            BleUuid::Uuid32(uuid) => write!(f, "0x{:04x}", uuid),
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