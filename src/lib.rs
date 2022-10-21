#[macro_use]
pub mod utilities;
pub mod gatt_server;

// TODO: Better log levels.
// TODO: Custom errors instead of panics.
// TODO: Clippy.
// TODO: R/W closures of characteristics should be generic in T, not just u8 slices.
// TODO: Remove references to pulse.loop.
//          - Fixed device appearance.
//          - Fixed advertising parameters and data.
//          - Fixed scan response data.
