//! This module contains useful structs and macros for the crate.

// Trait for converting data to little endian bytes: public.
mod le_bytes_serialisable;
pub use le_bytes_serialisable::LeBytesSerialisable;

// Leaky box: internally useful for ffi C interfacing: private.
pub(crate) mod leaky_box;

// Utilities: private
mod attribute_control;
pub(crate) use attribute_control::AttributeControl;

// Connection: private.
mod connection;
pub(crate) use connection::Connection;

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
