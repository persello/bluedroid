pub mod gatt_server;
#[macro_use]
pub mod utilities;

// TODO: Better log levels.
// TODO: Custom errors.
// TODO: Clippy.
// TODO: R/W closures of characteristics should be generic in T, not just u8 slices.
// TODO: Remove references to pulse.loop.
//          - Fixed device name.
//          - Fixed device appearance.
//          - Fixed advertising parameters and data.
//          - Fixed scan response data.