use crate::{
    gatt_server::Descriptor,
    utilities::{AttributePermissions, BleUuid},
};

impl Descriptor {
    pub fn user_description<S: AsRef<str>>(description: S) -> Self {
        Descriptor::new(
            "User Description",
            BleUuid::from_uuid16(0x2901),
            AttributePermissions::read(),
        )
        .set_value(description.as_ref().as_bytes().to_vec())
        .to_owned()
    }
    
    pub fn cccd() -> Self {
        Descriptor::new(
            "Client Characteristic Configuration",
                BleUuid::from_uuid16(0x2902),
                AttributePermissions::read_write(),
        )
        .set_value(0u16.to_le_bytes())
        .to_owned()
    }
}
