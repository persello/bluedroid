use std::ffi::c_char;

use crate::{
    gatt_server::GattServer,
    leaky_box_raw,
    utilities::{AttributeControl, BleUuid},
};
use esp_idf_sys::{
    esp_ble_gap_config_adv_data, esp_ble_gap_set_device_name, esp_ble_gap_start_advertising,
    esp_ble_gatts_cb_param_t, esp_ble_gatts_send_response, esp_ble_gatts_start_service,
    esp_bt_status_t_ESP_BT_STATUS_SUCCESS, esp_gatt_if_t, esp_gatt_rsp_t,
    esp_gatt_status_t_ESP_GATT_OK, esp_gatt_value_t, esp_gatts_cb_event_t,
    esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT, esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_CONNECT_EVT, esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT, esp_gatts_cb_event_t_ESP_GATTS_MTU_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_READ_EVT, esp_gatts_cb_event_t_ESP_GATTS_REG_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_RESPONSE_EVT, esp_gatts_cb_event_t_ESP_GATTS_START_EVT,
    esp_nofail,
};
use log::{debug, info, warn};

use crate::gatt_server::profile::Profile;

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

                // Do not pass this event to the profile handlers.
                return;
            }
            esp_gatts_cb_event_t_ESP_GATTS_DISCONNECT_EVT => {
                let param = unsafe { (*param).disconnect };
                info!(
                    "GATT client {:02X?} disconnected.",
                    param.remote_bda.to_vec()
                );

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
                        .iter_mut()
                        .find(|p| p.identifier == param.app_id)
                        .expect("No profile found with received identifier.");

                    profile.interface = Some(gatts_if);

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
            _ => {}
        }

        self.profiles.iter_mut().for_each(|profile| {
            if profile.interface == Some(gatts_if) {
                debug!("Handling event {} on profile {}.", event, profile);
                profile.gatts_event_handler(event, gatts_if, param)
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

                let service = self
                    .services
                    .iter_mut()
                    .find(|service| service.uuid == BleUuid::from(param.service_id.id))
                    .expect("Cannot find service described by received handle.");

                service.handle = Some(param.service_handle);

                if param.status != esp_gatt_status_t_ESP_GATT_OK {
                    warn!("GATT service registration failed.");
                } else {
                    info!(
                        "GATT service {} registered on handle 0x{:04x}.",
                        service,
                        service.handle.unwrap()
                    );

                    unsafe {
                        esp_nofail!(esp_ble_gatts_start_service(service.handle.unwrap()));
                    }

                    service.register_characteristics();
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_START_EVT => {
                let param = unsafe { (*param).start };

                let service = self
                    .services
                    .iter()
                    .find(|service| service.handle == Some(param.service_handle))
                    .expect("Cannot find service described by received handle.");

                if param.status != esp_gatt_status_t_ESP_GATT_OK {
                    warn!("GATT service {} failed to start.", service);
                } else {
                    debug!("GATT service {} started.", service);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT => {
                let param = unsafe { (*param).add_char };

                let characteristic = self
                    .services
                    .iter_mut()
                    .flat_map(|service| service.characteristics.iter_mut())
                    .find(|characteristic| characteristic.uuid == BleUuid::from(param.char_uuid))
                    .expect("Cannot find characteristic described by received UUID.");

                if param.status != esp_gatt_status_t_ESP_GATT_OK {
                    warn!("GATT characteristic registration failed.");
                } else {
                    info!(
                        "GATT characteristic {} registered at attribute handle 0x{:04x}.",
                        characteristic, param.attr_handle
                    );
                    characteristic.attribute_handle = Some(param.attr_handle);
                    characteristic.register_descriptors();
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT => {
                let param = unsafe { (*param).add_char_descr };

                let descriptor = self
                    .services
                    .iter_mut()
                    .flat_map(|service| service.characteristics.iter_mut())
                    .flat_map(|characteristic| characteristic.descriptors.iter_mut())
                    .find(|descriptor| descriptor.uuid == BleUuid::from(param.descr_uuid))
                    .expect("Cannot find descriptor described by received UUID.");

                if param.status != esp_gatt_status_t_ESP_GATT_OK {
                    warn!("GATT descriptor registration failed.");
                } else {
                    info!(
                        "GATT descriptor {} registered at attribute handle 0x{:04x}.",
                        descriptor, param.attr_handle
                    );
                    descriptor.attribute_handle = Some(param.attr_handle);
                }
            }
            esp_gatts_cb_event_t_ESP_GATTS_READ_EVT => {
                let param = unsafe { (*param).read };

                for service in self.services.iter_mut() {
                    for characteristic in service.characteristics.iter_mut() {
                        if characteristic.attribute_handle == Some(param.handle) {
                            debug!("Received read event for characteristic {}.", characteristic);

                            // If the characteristic has a read handler, call it.
                            if let AttributeControl::ResponseByApp(callback) =
                                characteristic.control
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
                            }
                        } else {
                            for descriptor in characteristic.descriptors.iter_mut() {
                                if descriptor.attribute_handle == Some(param.handle) {
                                    debug!("Received read event for descriptor {}.", descriptor);
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
