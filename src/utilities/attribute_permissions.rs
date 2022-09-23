use esp_idf_sys::*;

enum AttributeAccess {
    Read,
    Write,
    ReadWrite,
}

struct AttributePermissions {
    access: AttributeAccess,
    encryption_required: bool,
    authentication_required: bool,
    authorization_required: bool,
}

impl From<AttributePermissions> for esp_gatt_perm_t {
    fn from(permissions: AttributePermissions) -> Self {
        let mut result: u32 = 0;

        match permissions.access {
            AttributeAccess::Read => {
                result |= ESP_GATT_PERM_READ;
            }
            AttributeAccess::Write => {
                result |= ESP_GATT_PERM_WRITE;
            }
            AttributeAccess::ReadWrite => {
                result |= ESP_GATT_PERM_READ | ESP_GATT_PERM_WRITE;
            }
        }

        if permissions.encryption_required {
            result |= ESP_GATT_PERM_READ_ENCRYPTED;
        }

        result as Self
    }
}