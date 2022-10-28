use esp_idf_sys::*;

#[derive(Debug, Clone)]
pub enum AttributeControl {
    ResponseByApp(fn(esp_ble_gatts_cb_param_t_gatts_read_evt_param) -> Vec<u8>),
    AutomaticResponse(Vec<u8>),
}

impl From<AttributeControl> for esp_attr_control_t {
    fn from(control: AttributeControl) -> Self {
        let result: u8 = match control {
            AttributeControl::AutomaticResponse(_) => ESP_GATT_AUTO_RSP as u8,
            AttributeControl::ResponseByApp(_) => ESP_GATT_RSP_BY_APP as u8,
        };

        Self { auto_rsp: result }
    }
}
