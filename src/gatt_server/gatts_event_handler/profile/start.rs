use crate::gatt_server::Profile;
use esp_idf_sys::*;
use log::{debug, warn};

impl Profile {
    pub(crate) fn on_start(&mut self, param: esp_ble_gatts_cb_param_t_gatts_start_evt_param) {
        if let Some(service) = self.get_service(param.service_handle) {
            if param.status == esp_gatt_status_t_ESP_GATT_OK {
                debug!("GATT service {} started.", service.read().unwrap());
            } else {
                warn!("GATT service {} failed to start.", service.read().unwrap());
            }
        } else {
            warn!(
                "Cannot find service described by handle 0x{:04x} received in service start event.",
                param.service_handle
            );
        }
    }
}
