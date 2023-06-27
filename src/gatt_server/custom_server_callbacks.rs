use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_connect_evt_param,
    esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param,
};

type OnConnectCallback = dyn Fn(esp_ble_gatts_cb_param_t_gatts_connect_evt_param) + Send + Sync;
type OnDisconnectBallcack =
    dyn Fn(esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param) + Send + Sync;

pub(crate) struct CustomServerCallbacks {
    pub(crate) on_connect: Option<Box<OnConnectCallback>>,
    pub(crate) on_disconnect: Option<Box<OnDisconnectBallcack>>,
}

impl CustomServerCallbacks {
    pub(crate) fn on_connect(&self, param: esp_ble_gatts_cb_param_t_gatts_connect_evt_param) {
        if let Some(ref on_connect_callback) = self.on_connect {
            on_connect_callback(param)
        }
    }

    pub(crate) fn on_disconnect(&self, param: esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param) {
        if let Some(ref on_disconnect_callback) = self.on_disconnect {
            on_disconnect_callback(param)
        }
    }
}

impl Default for CustomServerCallbacks {
    fn default() -> Self {
        Self {
            on_connect: None,
            on_disconnect: None,
        }
    }
}
