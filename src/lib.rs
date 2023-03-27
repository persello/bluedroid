#![warn(missing_docs, unreachable_pub)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::multiple_crate_versions)]
// #![warn(clippy::std_instead_of_core)]
// #![warn(clippy::std_instead_of_alloc)]
// #![warn(clippy::alloc_instead_of_core)]
// #![warn(clippy::unwrap_in_result)]
// #![warn(clippy::unwrap_used)]
// #![warn(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

// In ESP32-S2, the Bluetooth controller is not present.
// Completely disable this crate.

#[cfg(not(esp32s2))]
pub mod gatt_server;

#[cfg(not(esp32s2))]
pub mod utilities;
