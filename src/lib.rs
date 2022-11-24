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

pub mod gatt_server;
pub mod utilities;

// TODO: Custom errors instead of panics.
// TODO: Builder pattern.
// TODO: Fix (no allow) some complex allows such as complexity and line limits.
// TODO: Characteristic presentation format.
