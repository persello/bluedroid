use std::{ffi::c_char};

use crate::{
    gatt_server::GattServer,
    leaky_box_raw,
    utilities::{AttributeControl, BleUuid},
};
use esp_idf_sys::{
    esp_ble_gap_config_adv_data, esp_ble_gap_set_device_name, esp_ble_gap_start_advertising,
    esp_ble_gatts_cb_param_t, esp_ble_gatts_send_indicate, esp_ble_gatts_send_response,
    esp_ble_gatts_start_service, esp_bt_status_t_ESP_BT_STATUS_SUCCESS, esp_gatt_if_t,
    esp_gatt_rsp_t, esp_gatt_status_t_ESP_GATT_OK, esp_gatt_value_t, esp_gatts_cb_event_t,
    esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT, esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_CONNECT_EVT, esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT, esp_gatts_cb_event_t_ESP_GATTS_MTU_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_READ_EVT, esp_gatts_cb_event_t_ESP_GATTS_REG_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_RESPONSE_EVT, esp_gatts_cb_event_t_ESP_GATTS_SET_ATTR_VAL_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_START_EVT, esp_gatts_cb_event_t_ESP_GATTS_WRITE_EVT, esp_nofail,
};
use log::{debug, info, warn};

use crate::gatt_server::profile::Profile;

use super::{characteristic, descriptor, Descriptor};

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
                info!("GATT client {:02X?} connected.", param.remote_bda.to_vec());
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
                        .find(|profile| (*profile).borrow().identifier == param.app_id)
                        .expect("No profile found with received application identifier.");

                    profile.borrow_mut().interface = Some(gatts_if);

                    if !self.advertisement_configured {
                        unsafe {
                            esp_nofail!(esp_ble_gap_set_device_name(
                                self.device_name.as_ptr() as *const c_char
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

                if let Some(profile) = self.get_profile(gatts_if) &&
                   let Some(service) = profile.borrow().get_service(param.srvc_handle) &&
                   let Some(characteristic) = service.borrow().get_characteristic(param.attr_handle) {
                        debug!(
                            "Received set attribute value event for characteristic {}.",
                            characteristic.borrow()
                        );

                        if characteristic.borrow().properties.indicate {
                            for connection in self.active_connections.clone() {
                                unsafe {
                                    esp_nofail!(esp_ble_gatts_send_indicate(
                                        gatts_if,
                                        connection.id(),
                                        param.attr_handle,
                                        characteristic.borrow().internal_value.len() as u16,
                                        characteristic.borrow_mut().internal_value.as_mut_slice().as_mut_ptr(),
                                        false
                                    ));
                                }
                            }
                        } else if characteristic.borrow().properties.notify {
                            for connection in self.active_connections.clone() {
                                unsafe {
                                    esp_nofail!(esp_ble_gatts_send_indicate(
                                        gatts_if,
                                        connection.id(),
                                        param.attr_handle,
                                        characteristic.borrow().internal_value.len() as u16,
                                        characteristic.borrow_mut().internal_value.as_mut_slice().as_mut_ptr(),
                                        true
                                    ));
                                }
                            }
                        }
                    } else {
                        warn!("Cannot find characteristic described by service handle {} and attribute handle {} received in set attribute value event.", param.srvc_handle, param.attr_handle);
                    }
            }
            _ => {}
        }

        self.profiles.iter().for_each(|profile| {
            if profile.borrow().interface == Some(gatts_if) {
                debug!("Handling event {} on profile {}.", event, profile.borrow());
                profile
                    .borrow_mut()
                    .gatts_event_handler(event, gatts_if, param)
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
                if param.status != esp_bt_status_t_ESP_BT_STATUS_SUCCESS {
                    warn!("GATT profile registration failed.");
                } else {
                    info!(
                        "{} registered on interface {}.",
                        &self,
                        self.interface.unwrap()
                    );
                    self.register_services();
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT => {
                let param = unsafe { (*param).create };

                if let Some(service) = self.get_service_by_id(param.service_id.id) {
                    service.borrow_mut().handle = Some(param.service_handle);

                    if param.status != esp_gatt_status_t_ESP_GATT_OK {
                        warn!("GATT service registration failed.");
                    } else {
                        info!(
                            "GATT service {} registered on handle 0x{:04x}.",
                            service.borrow(),
                            service.borrow().handle.unwrap()
                        );

                        unsafe {
                            esp_nofail!(esp_ble_gatts_start_service(
                                service.borrow().handle.unwrap()
                            ));
                        }

                        service.borrow_mut().register_characteristics();
                    }
                } else {
                    warn!("Cannot find service with service identifier {} received in service creation event.", BleUuid::from(param.service_id.id));
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_START_EVT => {
                let param = unsafe { (*param).start };

                if let Some(service) = self.get_service(param.service_handle) {
                    if param.status != esp_gatt_status_t_ESP_GATT_OK {
                        warn!("GATT service {} failed to start.", service.borrow());
                    } else {
                        debug!("GATT service {} started.", service.borrow());
                    }
                } else {
                    warn!("Cannot find service described by handle 0x{:04x} received in service start event.", param.service_handle);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT => {
                let param = unsafe { (*param).add_char };

                if let Some(service) = self.get_service(param.service_handle) &&
                   let Some(characteristic) = service.borrow().get_characteristic_by_id(param.char_uuid) {
                        if param.status != esp_gatt_status_t_ESP_GATT_OK {
                            warn!("GATT characteristic registration failed.");
                        } else {
                            info!(
                                "GATT characteristic {} registered at attribute handle 0x{:04x}.",
                                characteristic.borrow(),
                                param.attr_handle
                            );
                            characteristic.borrow_mut().attribute_handle = Some(param.attr_handle);
                            characteristic.borrow_mut().register_descriptors();
                        }
                } else {
                    warn!("Cannot find characteristic described by service handle 0x{:04x} and characteristic identifier {} received in characteristic creation event.", param.service_handle, BleUuid::from(param.char_uuid));
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT => {
                let param = unsafe { (*param).add_char_descr };

                if let Some(service) = self.get_service(param.service_handle) &&
                   let Some(descriptor) = service.borrow().get_descriptor_by_id(param.descr_uuid)
                {
                    if param.status != esp_gatt_status_t_ESP_GATT_OK {
                        warn!("GATT descriptor registration failed.");
                    } else {
                        info!(
                            "GATT descriptor {} registered at attribute handle 0x{:04x}.",
                            descriptor.borrow(),
                            param.attr_handle
                        );
                        descriptor.borrow_mut().attribute_handle = Some(param.attr_handle);
                    }
                } else {
                    warn!("Cannot find service described by identifier {} received in descriptor creation event.", BleUuid::from(param.descr_uuid));
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_WRITE_EVT => {
                let param = unsafe { (*param).write };

                for service in self.services.iter() {
                    for characteristic in service.borrow().characteristics.iter() {
                        if characteristic.borrow().attribute_handle == Some(param.handle) {
                            debug!(
                                "Received write event for characteristic {}.",
                                characteristic.borrow()
                            );

                            // If the characteristic has a write handler, call it.
                            if let Some(write_callback) = characteristic.borrow().write_callback {
                                let value = unsafe {
                                    std::slice::from_raw_parts(param.value, param.len as usize)
                                }
                                .to_vec();

                                write_callback(value);

                                // Send response if needed.
                                if param.need_rsp && let AttributeControl::ResponseByApp(read_callback) = characteristic.borrow().control {
                                    // Get value.
                                    let value = read_callback();

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
                                return;
                            }
                        }
                    }
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_READ_EVT => {
                let param = unsafe { (*param).read };

                for service in self.services.iter() {
                    for characteristic in service.borrow().characteristics.iter() {
                        if characteristic.borrow().attribute_handle == Some(param.handle) {
                            debug!(
                                "Received read event for characteristic {}.",
                                characteristic.borrow()
                            );

                            // If the characteristic has a read handler, call it.
                            if let AttributeControl::ResponseByApp(callback) =
                                characteristic.borrow().control
                            {
                                let value = callback();

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

                                return;
                            }
                        } else {
                            for descriptor in characteristic.borrow().descriptors.iter() {
                                if descriptor.borrow().attribute_handle == Some(param.handle) {
                                    debug!(
                                        "Received read event for descriptor {}.",
                                        descriptor.borrow()
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                warn!("Unhandled GATT server event: {:?}", event);
            }
        }
    }
}
