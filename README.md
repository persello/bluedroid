# Bluedroid Rust wrapper

[![crates.io](https://img.shields.io/crates/v/bluedroid)](https://crates.io/crates/bluedroid)
[![docs.rs](https://docs.rs/bluedroid/badge.svg)](https://docs.rs/bluedroid)
![crates.io](https://img.shields.io/crates/d/bluedroid)
![crates.io](https://img.shields.io/crates/l/bluedroid)

This is a Rust wrapper for the Bluedroid Bluetooth stack for ESP32.
It allows you to build a GATT server with a declarative API and supports multithreading.

## Usage

Declare a characteristic:

```rust
let manufacturer_name_characteristic = Characteristic::new(BleUuid::Uuid16(0x2A29))
      .name("Manufacturer Name String")
      .permissions(AttributePermissions::new().read().write())
      .properties(CharacteristicProperties::new().read().write().notify())
      .max_value_length(20)
      .on_write(|data, param| {
          info!("Received write request: {:?} {:?}", data, param);
      })
      .show_name()
      .set_value("Hello, world!".as_bytes().to_vec())
      .build();
```

Declare a service:

```rust
let device_information_service = Service::new(BleUuid::Uuid16(0x180A))
    .name("Device Information")
    .primary()
    .characteristic(&manufacturer_name_characteristic)
    .build();
```

Declare a profile and start the server:

```rust
let profile = Profile::new(0x0001)
    .name("Device Information")
    .service(&device_information_service)
    .build();

GLOBAL_GATT_SERVER
    .lock()
    .unwrap()
    .profile(profile)
    .device_name("ESP32-GATT-Server")
    .appearance(Appearance::WristWornPulseOximeter)
    .advertise_service(&device_information_service)
    .start();
```

## Features

- [x] GATT server
  - [x] Advertisement
    - [x] Custom name
    - [x] Custom appearance
  - [x] Multiple applications
  - [x] Services
    - [x] Declaration
    - [x] Advertisement
  - [x] Characteristics
    - [x] Declaration
    - [x] Broadcast
    - [x] Read
      - [x] Static (by stack)
      - [x] Dynamic (by application, with callback)
      - [ ] Long
    - [x] Write
      - [x] With response
      - [x] Without response
      - [ ] Long
    - [x] Notify
    - [x] Indicate
  - [x] Descriptors
    - [x] Declaration
    - [x] Read
    - [x] Write
  - [ ] Encryption
- [ ] GATT client
  > There are currently no plans to implement the GATT client API.
  > Contributions are welcome.
- [ ] BR/EDR
  > There are currently no plans to implement the Bluetooth Classic API.
  > Contributions are welcome.
