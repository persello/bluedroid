// Leaky box: internally useful for ffi C interfacing.
pub(crate) mod leaky_box;

// BLE identifiers: public.
mod ble_uuid;
pub use ble_uuid::BleUuid;

// Bluetooth device appearance: public.
mod appearance;
pub use appearance::Appearance;

// Characteristic properties: public.
mod characteristic_properties;
pub use characteristic_properties::CharacteristicProperties;

// Attribute permissions: public.
mod attribute_permissions;
pub use attribute_permissions::AttributePermissions;

// Attribute control: public.
mod attribute_control;
pub use attribute_control::AttributeControl;

// Connection: public.
mod connection;
pub use connection::Connection;
