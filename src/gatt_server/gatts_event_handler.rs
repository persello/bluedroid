#![allow(clippy::too_many_lines)]

use crate::{
    gatt_server::{GattServer, Profile},
    leaky_box_raw,
    utilities::{AttributeControl, BleUuid, Connection},
};

use esp_idf_sys::{
    esp_ble_gap_config_adv_data, esp_ble_gap_set_device_name, esp_ble_gap_start_advertising,
    esp_ble_gatts_cb_param_t, esp_ble_gatts_cb_param_t_gatts_read_evt_param,
    esp_ble_gatts_get_attr_value, esp_ble_gatts_send_indicate, esp_ble_gatts_send_response,
    esp_ble_gatts_start_service, esp_bt_status_t_ESP_BT_STATUS_SUCCESS, esp_gatt_if_t,
    esp_gatt_rsp_t, esp_gatt_status_t_ESP_GATT_OK, esp_gatt_value_t, esp_gatts_cb_event_t,
    esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT, esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_CONF_EVT, esp_gatts_cb_event_t_ESP_GATTS_CONNECT_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT, esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_MTU_EVT, esp_gatts_cb_event_t_ESP_GATTS_READ_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_REG_EVT, esp_gatts_cb_event_t_ESP_GATTS_RESPONSE_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_SET_ATTR_VAL_EVT, esp_gatts_cb_event_t_ESP_GATTS_START_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_WRITE_EVT, esp_nofail,
};
use log::{debug, info, warn};

impl GattServer {
    /// The main GATT server event loop.
    ///
    /// Dispatches the received events across the appropriate profile-related handlers.
    pub(crate) fn gatts_event_handler(
        &mut self,
        event: esp_gatts_cb_event_t,
        gatts_if: esp_gatt_if_t,
        param: *mut esp_ble_gatts_cb_param_t,
    ) {
        #[allow(non_upper_case_globals)]
        match event {
            esp_gatts_cb_event_t_ESP_GATTS_CONNECT_EVT => {
                let param = unsafe { (*param).connect };
                info!("GATT client {} connected.", Connection::from(param));
                self.active_connections.insert(param.into());

                // Do not pass this event to the profile handlers.
                return;
            }
            esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT => {
                let param = unsafe { (*param).disconnect };
                info!(
                    "GATT client {:02X?} disconnected.",
                    param.remote_bda.to_vec()
                );

                self.active_connections.remove(&param.into());

                unsafe {
                    esp_ble_gap_start_advertising(leaky_box_raw!(self.advertisement_parameters));
                }

                // Do not pass this event to the profile handlers.
                return;
            }
            esp_gatts_cb_event_t_ESP_GATTS_MTU_EVT => {
                let param = unsafe { (*param).mtu };
                debug!("MTU changed to {}.", param.mtu);

                // Do not pass this event to the profile handlers.
                return;
            }
            esp_gatts_cb_event_t_ESP_GATTS_REG_EVT => {
                let param = unsafe { (*param).reg };
                if param.status == esp_gatt_status_t_ESP_GATT_OK {
                    debug!("New profile registered.");

                    let profile = self
                        .profiles
                        .iter()
                        .find(|profile| (*profile).read().unwrap().identifier == param.app_id)
                        .expect("No profile found with received application identifier.");

                    profile.write().unwrap().interface = Some(gatts_if);

                    if !self.advertisement_configured {
                        unsafe {
                            esp_nofail!(esp_ble_gap_set_device_name(
                                self.device_name.as_ptr().cast::<i8>()
                            ));

                            self.advertisement_configured = true;

                            // Advertisement data.
                            esp_nofail!(esp_ble_gap_config_adv_data(leaky_box_raw!(
                                self.advertisement_data
                            )));

                            // Scan response data.
                            esp_nofail!(esp_ble_gap_config_adv_data(leaky_box_raw!(
                                self.scan_response_data
                            )));
                        }
                    }
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_RESPONSE_EVT => {
                let param = unsafe { (*param).rsp };

                debug!("Responded to handle 0x{:04x}.", param.handle);

                // Do not pass this event to the profile handlers.
                return;
            }
            esp_gatts_cb_event_t_ESP_GATTS_SET_ATTR_VAL_EVT => {
                let param = unsafe { (*param).set_attr_val };

                if param.status != esp_gatt_status_t_ESP_GATT_OK {
                    warn!(
                        "Failed to set attribute value, error code: {:04x}.",
                        param.status
                    );
                }

                if let Some(profile) = self.get_profile(gatts_if) {
                    if let Some(service) = profile.read().unwrap().get_service(param.srvc_handle) {
                        if let Some(characteristic) = service
                            .read()
                            .unwrap()
                            .get_characteristic_by_handle(param.attr_handle)
                        {
                            debug!(
                                "Received set attribute value event for characteristic {}.",
                                characteristic.read().unwrap()
                            );

                            if characteristic.read().unwrap().properties.indicate {
                                for connection in self.active_connections.clone() {
                                    let simulated_read_param =
                                        esp_ble_gatts_cb_param_t_gatts_read_evt_param {
                                            bda: connection.remote_bda,
                                            conn_id: connection.id,
                                            handle: characteristic
                                                .read()
                                                .unwrap()
                                                .descriptors
                                                .iter()
                                                .find(|desc| {
                                                    desc.read().unwrap().uuid
                                                        == BleUuid::Uuid16(0x2902)
                                                })
                                                .unwrap()
                                                .read()
                                                .unwrap()
                                                .attribute_handle
                                                .unwrap(),
                                            ..Default::default()
                                        };

                                    let status = characteristic
                                        .read()
                                        .unwrap()
                                        .get_cccd_status(simulated_read_param);

                                    if let Some((_, indication)) = status {
                                        if indication {
                                            debug!(
                                                "Indicating {} value change to {:02X?}.",
                                                characteristic.read().unwrap(),
                                                connection.id
                                            );
                                            let mut internal_value = characteristic
                                                .write()
                                                .unwrap()
                                                .internal_value
                                                .clone();
                                            unsafe {
                                                esp_nofail!(esp_ble_gatts_send_indicate(
                                                    gatts_if,
                                                    connection.id,
                                                    param.attr_handle,
                                                    internal_value.len() as u16,
                                                    internal_value.as_mut_slice().as_mut_ptr(),
                                                    true
                                                ));
                                            }
                                        }
                                    }
                                }
                            } else if characteristic.read().unwrap().properties.notify {
                                for connection in self.active_connections.clone() {
                                    let simulated_read_param =
                                        esp_ble_gatts_cb_param_t_gatts_read_evt_param {
                                            bda: connection.remote_bda,
                                            conn_id: connection.id,
                                            handle: characteristic
                                                .read()
                                                .unwrap()
                                                .descriptors
                                                .iter()
                                                .find(|desc| {
                                                    desc.read().unwrap().uuid
                                                        == BleUuid::Uuid16(0x2902)
                                                })
                                                .unwrap()
                                                .read()
                                                .unwrap()
                                                .attribute_handle
                                                .unwrap(),
                                            ..Default::default()
                                        };

                                    let status = characteristic
                                        .read()
                                        .unwrap()
                                        .get_cccd_status(simulated_read_param);

                                    if let Some((notification, _)) = status {
                                        if notification {
                                            debug!(
                                                "Notifying {} value change to {}.",
                                                characteristic.read().unwrap(),
                                                connection
                                            );
                                            let mut internal_value = characteristic
                                                .write()
                                                .unwrap()
                                                .internal_value
                                                .clone();
                                            unsafe {
                                                esp_nofail!(esp_ble_gatts_send_indicate(
                                                    gatts_if,
                                                    connection.id,
                                                    param.attr_handle,
                                                    internal_value.len() as u16,
                                                    internal_value.as_mut_slice().as_mut_ptr(),
                                                    false
                                                ));
                                            }
                                        }
                                    }
                                }
                            }

                            let value: *mut *const u8 = &mut [0u8].as_ptr();
                            let mut len = 512;
                            let vector = unsafe {
                                esp_nofail!(esp_ble_gatts_get_attr_value(
                                    param.attr_handle,
                                    &mut len,
                                    value,
                                ));

                                std::slice::from_raw_parts(*value, len as usize)
                            };

                            debug!(
                                "Characteristic {} value changed to {:02X?}.",
                                characteristic.read().unwrap(),
                                vector
                            );
                        } else {
                            warn!("Cannot find characteristic described by service handle {} and attribute handle {} received in set attribute value event.", param.srvc_handle, param.attr_handle);
                        }
                    } else {
                        warn!("Cannot find service described by service handle {} received in set attribute value event.", param.srvc_handle);
                    }
                } else {
                    warn!("Cannot find profile described by interface {} received in set attribute value event.", gatts_if);
                }

                // Do not pass this event to the profile handlers.
                return;
            }
            _ => {}
        }

        self.profiles.iter().for_each(|profile| {
            if profile.read().unwrap().interface == Some(gatts_if) {
                debug!(
                    "Handling event {} on profile {}.",
                    event,
                    profile.read().unwrap()
                );
                profile
                    .write()
                    .unwrap()
                    .gatts_event_handler(event, gatts_if, param);
            }
        });
    }
}

impl Profile {
    /// Profile-specific GATT server event loop.
    fn gatts_event_handler(
        &mut self,
        event: esp_gatts_cb_event_t,
        gatts_if: esp_gatt_if_t,
        param: *mut esp_ble_gatts_cb_param_t,
    ) {
        #[allow(non_upper_case_globals)]
        match event {
            esp_gatts_cb_event_t_ESP_GATTS_REG_EVT => {
                let param = unsafe { (*param).reg };

                // Check status
                if param.status == esp_bt_status_t_ESP_BT_STATUS_SUCCESS {
                    info!(
                        "{} registered on interface {}.",
                        &self,
                        self.interface.unwrap()
                    );
                    self.register_services();
                } else {
                    warn!("GATT profile registration failed.");
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT => {
                let param = unsafe { (*param).create };

                if let Some(service) = self.get_service_by_id(param.service_id.id) {
                    service.write().unwrap().handle = Some(param.service_handle);

                    if param.status == esp_gatt_status_t_ESP_GATT_OK {
                        info!(
                            "GATT service {} registered on handle 0x{:04x}.",
                            service.read().unwrap(),
                            service.read().unwrap().handle.unwrap()
                        );

                        unsafe {
                            esp_nofail!(esp_ble_gatts_start_service(
                                service.read().unwrap().handle.unwrap()
                            ));
                        }

                        service.write().unwrap().register_characteristics();
                    } else {
                        warn!("GATT service registration failed.");
                    }
                } else {
                    warn!("Cannot find service with service identifier {} received in service creation event.", BleUuid::from(param.service_id.id));
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_START_EVT => {
                let param = unsafe { (*param).start };

                if let Some(service) = self.get_service(param.service_handle) {
                    if param.status == esp_gatt_status_t_ESP_GATT_OK {
                        debug!("GATT service {} started.", service.read().unwrap());
                    } else {
                        warn!("GATT service {} failed to start.", service.read().unwrap());
                    }
                } else {
                    warn!("Cannot find service described by handle 0x{:04x} received in service start event.", param.service_handle);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT => {
                let param = unsafe { (*param).add_char };

                if let Some(service) = self.get_service(param.service_handle) {
                    if let Some(characteristic) = service
                        .read()
                        .unwrap()
                        .get_characteristic_by_id(param.char_uuid)
                    {
                        if param.status == esp_gatt_status_t_ESP_GATT_OK {
                            info!(
                                "GATT characteristic {} registered at attribute handle 0x{:04x}.",
                                characteristic.read().unwrap(),
                                param.attr_handle
                            );
                            characteristic.write().unwrap().attribute_handle =
                                Some(param.attr_handle);
                            characteristic.write().unwrap().register_descriptors();
                        } else {
                            warn!("GATT characteristic registration failed.");
                        }
                    } else {
                        warn!("Cannot find characteristic described by service handle 0x{:04x} and characteristic identifier {} received in characteristic creation event.", param.service_handle, BleUuid::from(param.char_uuid));
                    }
                } else {
                    warn!("Cannot find service described by handle 0x{:04x} received in characteristic creation event.", param.service_handle);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT => {
                let param = unsafe { (*param).add_char_descr };

                // ATTENTION: Descriptors might have duplicate UUIDs!
                // We need to set them in order of creation.

                if let Some(service) = self.get_service(param.service_handle) {
                    if let Some(descriptor) = service
                        .read()
                        .unwrap()
                        .get_descriptors_by_id(param.descr_uuid)
                        .iter()
                        .find(|d| d.read().unwrap().attribute_handle.is_none())
                    {
                        if param.status == esp_gatt_status_t_ESP_GATT_OK {
                            info!(
                                "GATT descriptor {} registered at attribute handle 0x{:04x}.",
                                descriptor.read().unwrap(),
                                param.attr_handle
                            );
                            descriptor.write().unwrap().attribute_handle = Some(param.attr_handle);
                        } else {
                            warn!("GATT descriptor registration failed.");
                        }
                    } else {
                        warn!("Cannot find service described by identifier {} received in descriptor creation event.", BleUuid::from(param.descr_uuid));
                    }
                } else {
                    warn!("Cannot find service described by handle 0x{:04x} received in descriptor creation event.", param.service_handle);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_WRITE_EVT => {
                let param = unsafe { (*param).write };

                for service in &self.services {
                    service.read().unwrap().characteristics.iter().for_each(|characteristic| {
                        if characteristic.read().unwrap().attribute_handle == Some(param.handle) {
                            debug!(
                                "Received write event for characteristic {}.",
                                characteristic.read().unwrap()
                            );

                            // If the characteristic has a write handler, call it.
                            if let Some(write_callback) =
                                &characteristic.read().unwrap().write_callback
                            {
                                let value = unsafe {
                                    std::slice::from_raw_parts(param.value, param.len as usize)
                                }
                                .to_vec();

                                write_callback(value, param);

                                // Send response if needed.
                                if param.need_rsp {
                                    if let AttributeControl::ResponseByApp(read_callback) = &characteristic.read().unwrap().control {

                                    // Simulate a read operation.
                                    let param_as_read_operation = esp_ble_gatts_cb_param_t_gatts_read_evt_param {
                                        bda: param.bda,
                                        conn_id: param.conn_id,
                                        handle: param.handle,
                                        need_rsp: param.need_rsp,
                                        offset: param.offset,
                                        trans_id: param.trans_id,
                                        ..Default::default()
                                    };

                                    // Get value.
                                    let value = read_callback(param_as_read_operation);

                                    // Extend the response to the maximum length.
                                    let mut response = [0u8; 600];
                                    response[..value.len()].copy_from_slice(&value);

                                    unsafe {
                                        esp_nofail!(esp_ble_gatts_send_response(
                                            gatts_if,
                                            param.conn_id,
                                            param.trans_id,
                                            esp_gatt_status_t_ESP_GATT_OK,
                                            leaky_box_raw!(esp_gatt_rsp_t {
                                                attr_value: esp_gatt_value_t {
                                                    auth_req: 0,
                                                    handle: param.handle,
                                                    len: value.len() as u16,
                                                    offset: 0,
                                                    value: response,
                                                },
                                            })
                                        ));
                                    }
                                }
                            }
                            }
                        } else {
                            characteristic.read().unwrap().descriptors.iter().for_each(
                                |descriptor| {
                                    if descriptor.read().unwrap().attribute_handle == Some(param.handle)
                                    {
                                        debug!(
                                            "Received write event for descriptor {}.",
                                            descriptor.read().unwrap()
                                        );

                                        if let Some(write_callback) = descriptor.read().unwrap().write_callback {
                                            let value = unsafe {
                                                std::slice::from_raw_parts(param.value, param.len as usize)
                                            }
                                            .to_vec();

                                            write_callback(value, param);

                                            // Send response if needed.
                                            if param.need_rsp {
                                                if let AttributeControl::ResponseByApp(read_callback) = &descriptor.read().unwrap().control {

                                                // Simulate a read operation.
                                                let param_as_read_operation = esp_ble_gatts_cb_param_t_gatts_read_evt_param {
                                                    bda: param.bda,
                                                    conn_id: param.conn_id,
                                                    handle: param.handle,
                                                    need_rsp: param.need_rsp,
                                                    offset: param.offset,
                                                    trans_id: param.trans_id,
                                                    ..Default::default()
                                                };

                                                // Get value.
                                                let value = read_callback(param_as_read_operation);

                                                // Extend the response to the maximum length.
                                                let mut response = [0u8; 600];
                                                response[..value.len()].copy_from_slice(&value);

                                                unsafe {
                                                    esp_nofail!(esp_ble_gatts_send_response(
                                                        gatts_if,
                                                        param.conn_id,
                                                        param.trans_id,
                                                        esp_gatt_status_t_ESP_GATT_OK,
                                                        leaky_box_raw!(esp_gatt_rsp_t {
                                                            attr_value: esp_gatt_value_t {
                                                                auth_req: 0,
                                                                handle: param.handle,
                                                                len: value.len() as u16,
                                                                offset: 0,
                                                                value: response,
                                                            },
                                                        })
                                                    ));
                                                }
                                            }
                                        }
                                        }
                                    }
                                },
                            );
                        }
                    });
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_READ_EVT => {
                let param = unsafe { (*param).read };

                for service in &self.services {
                    service
                        .read()
                        .unwrap()
                        .characteristics
                        .iter()
                        .for_each(|characteristic| {
                            debug!(
                                "MCC: Checking characteristic {} ({:?}).",
                                characteristic.read().unwrap(),
                                characteristic.read().unwrap().attribute_handle
                            );

                            if characteristic.read().unwrap().attribute_handle == Some(param.handle)
                            {
                                debug!(
                                    "Received read event for characteristic {}.",
                                    characteristic.read().unwrap()
                                );

                                // If the characteristic has a read handler, call it.
                                if let AttributeControl::ResponseByApp(callback) =
                                    &characteristic.read().unwrap().control
                                {
                                    let value = callback(param);

                                    // Extend the response to the maximum length.
                                    let mut response = [0u8; 600];
                                    response[..value.len()].copy_from_slice(&value);

                                    unsafe {
                                        esp_nofail!(esp_ble_gatts_send_response(
                                            gatts_if,
                                            param.conn_id,
                                            param.trans_id,
                                            // TODO: Allow different statuses.
                                            esp_gatt_status_t_ESP_GATT_OK,
                                            leaky_box_raw!(esp_gatt_rsp_t {
                                                attr_value: esp_gatt_value_t {
                                                    auth_req: 0,
                                                    handle: param.handle,
                                                    len: value.len() as u16,
                                                    offset: 0,
                                                    value: response,
                                                },
                                            })
                                        ));
                                    }
                                }
                            } else {
                                characteristic.read().unwrap().descriptors.iter().for_each(
                                    |descriptor| {
                                        debug!(
                                            "MCC: Checking descriptor {} ({:?}).",
                                            descriptor.read().unwrap(),
                                            descriptor.read().unwrap().attribute_handle
                                        );

                                        if descriptor.read().unwrap().attribute_handle
                                            == Some(param.handle)
                                        {
                                            debug!(
                                                "Received read event for descriptor {}.",
                                                descriptor.read().unwrap()
                                            );

                                            if let AttributeControl::ResponseByApp(callback) =
                                                &descriptor.read().unwrap().control
                                            {
                                                let value = callback(param);

                                                // Extend the response to the maximum length.
                                                let mut response = [0u8; 600];
                                                response[..value.len()].copy_from_slice(&value);

                                                unsafe {
                                                    esp_nofail!(esp_ble_gatts_send_response(
                                                        gatts_if,
                                                        param.conn_id,
                                                        param.trans_id,
                                                        esp_gatt_status_t_ESP_GATT_OK,
                                                        leaky_box_raw!(esp_gatt_rsp_t {
                                                            attr_value: esp_gatt_value_t {
                                                                auth_req: 0,
                                                                handle: param.handle,
                                                                len: value.len() as u16,
                                                                offset: 0,
                                                                value: response,
                                                            },
                                                        })
                                                    ));
                                                }
                                            }
                                        }
                                    },
                                );
                            }
                        });
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_CONF_EVT => {
                // let param = unsafe { (*param).conf };

                debug!("Received confirmation event.");
            }
            _ => {
                warn!("Unhandled GATT server event: {:?}", event);
            }
        }
    }
}
