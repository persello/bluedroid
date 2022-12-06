use esp_idf_sys::*;
use std::sync::Arc;

use super::LeBytesSerialisable;

#[derive(Clone)]
pub(crate) enum AttributeControl<T: LeBytesSerialisable> {
    ResponseByApp(
        Arc<dyn Fn(esp_ble_gatts_cb_param_t_gatts_read_evt_param) -> T + Send + Sync>,
    ),
    AutomaticResponse(T),
}

impl<T> From<AttributeControl<T>> for esp_attr_control_t {
    fn from(control: AttributeControl<T>) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let result: u8 = match control {
            AttributeControl::AutomaticResponse(_) => ESP_GATT_AUTO_RSP as u8,
            AttributeControl::ResponseByApp(_) => ESP_GATT_RSP_BY_APP as u8,
        };

        Self { auto_rsp: result }
    }
}

impl<T> std::fmt::Debug for AttributeControl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeControl::AutomaticResponse(_) => write!(f, "automatic response"),
            AttributeControl::ResponseByApp(_) => write!(f, "response by app"),
        }
    }
}
