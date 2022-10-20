use crate::{gatt_server::Descriptor, utilities::{BleUuid, AttributePermissions, CharacteristicProperties, AttributeControl}};

use super::Characteristic;

impl Descriptor {
    pub fn user_description<S: AsRef<str>>(description: S) -> Self {
        Descriptor::new(
            "User Description",
            BleUuid::from_uuid16(0x2901),
            AttributePermissions::read(),
        ).set_value(description.as_ref().as_bytes().to_vec()).to_owned()
    }
}
