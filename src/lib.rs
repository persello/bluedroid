#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::multiple_crate_versions)]

#[macro_use]
pub mod utilities;
pub mod gatt_server;

// TODO: Better log levels.
// TODO: Custom errors instead of panics.
// TODO: Clippy.
// TODO: Builder pattern.
// TODO: Remove some complex allows such as complexity and line limits.
