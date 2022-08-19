use crate::{gatt_server::GattServer, leaky_box_raw, utilities::ble_uuid::BleUuid};
use esp_idf_sys::{
    esp_ble_gap_config_adv_data, esp_ble_gap_set_device_name, esp_ble_gatts_cb_param_t,
    esp_ble_gatts_start_service, esp_bt_status_t_ESP_BT_STATUS_SUCCESS, esp_gatt_if_t,
    esp_gatt_status_t_ESP_GATT_OK, esp_gatts_cb_event_t,
    esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_DESCR_EVT, esp_gatts_cb_event_t_ESP_GATTS_ADD_CHAR_EVT,
    esp_gatts_cb_event_t_ESP_GATTS_CREATE_EVT, esp_gatts_cb_event_t_ESP_GATTS_REG_EVT, esp_nofail,
};
use log::{info, warn};

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
        if event == esp_gatts_cb_event_t_ESP_GATTS_REG_EVT {
            let param = unsafe { (*param).reg };
            if param.status == esp_gatt_status_t_ESP_GATT_OK {
                info!("New profile registered. Setting GAP device name.");

                let profile = self
                    .profiles
                    .iter_mut()
                    .find(|p| p.identifier == param.app_id)
                    .expect("No profile found with received identifier.");

                profile.interface = Some(gatts_if);         // CRASH HERE

                unsafe {
                    esp_nofail!(esp_ble_gap_set_device_name(
                        // TODO: Update name.
                        b"ESP32-GATT-Server\0".as_ptr() as *const _,
                    ));

                    // Advertisement data.
                    esp_nofail!(esp_ble_gap_config_adv_data(leaky_box_raw!(
                        self.advertisement_data
                    )));

                    // Scan response data.
                    esp_nofail!(esp_ble_gap_config_adv_data(leaky_box_raw!(
                        esp_idf_sys::esp_ble_adv_data_t {
                            set_scan_rsp: true,
                            ..self.advertisement_data
                        }
                    )));
                }
            }
        }

        self.profiles.iter_mut().for_each(|profile| {
            if profile.interface == Some(gatts_if) {
                info!("Handling event {} on profile {}.", event, profile);
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
                    info!("{} registered on interface {}.", &self, self.interface.unwrap());
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
                        "GATT service {} registered on handle {}.",
                        service,
                        service.handle.unwrap()
                    );

                    unsafe {
                        esp_nofail!(esp_ble_gatts_start_service(service.handle.unwrap()));
                    }

                    service.register_characteristics();
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
                    info!("GATT characteristic {} registered.", characteristic);
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
                    info!("GATT descriptor {} registered.", descriptor);
                }
            }
            _ => {
                warn!("Unhandled GATT server event: {:?}", event);
            }
        }
    }
}
