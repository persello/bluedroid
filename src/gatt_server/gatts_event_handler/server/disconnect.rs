use crate::gatt_server::GattServer;
use log::info;

impl GattServer {
    pub(crate) fn on_disconnect(
        &mut self,
        param: esp_idf_sys::esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param,
    ) {
        info!(
            "GATT client {:02X?} disconnected.",
            param.remote_bda.to_vec()
        );

        self.active_connections.remove(&param.into());
        self.custom_server_callbacks.on_disconnect(param);

        unsafe {
            esp_idf_sys::esp_ble_gap_start_advertising(&mut self.advertisement_parameters);
        }
    }
}
