use crate::gatt_server::GattServer;
use esp_idf_sys::*;

impl GattServer {
    fn gatts_event_handler(
        &mut self,
        event: esp_gatts_cb_event_t,
        gatts_if: esp_gatt_if_t,
        param: *mut esp_ble_gatts_cb_param_t,
    ) {
        let params = unsafe { (*param).reg };
        if event == esp_gatts_cb_event_t_ESP_GATTS_REG_EVT && params.status == esp_gatt_status_t_ESP_GATT_OK {
            // self.applications
        }
    }
}
