use esp_idf_sys::{
    esp_ble_gatts_cb_param_t_gatts_connect_evt_param,
    esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param,
};

#[derive(Debug, Copy, Clone)]
pub struct Connection {
    id: u16,
    is_slave: bool,
    remote_bda: [u8; 6],
}

impl Connection {
    pub fn new(id: u16, is_slave: bool, remote_bda: [u8; 6]) -> Self {
        Self {
            id,
            is_slave,
            remote_bda,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn is_slave(&self) -> bool {
        self.is_slave
    }

    pub fn remote_bda(&self) -> [u8; 6] {
        self.remote_bda
    }
}

impl From<esp_ble_gatts_cb_param_t_gatts_connect_evt_param> for Connection {
    fn from(param: esp_ble_gatts_cb_param_t_gatts_connect_evt_param) -> Self {
        Self {
            id: param.conn_id,
            is_slave: param.link_role == 1,
            remote_bda: param.remote_bda,
        }
    }
}

impl From<esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param> for Connection {
    fn from(param: esp_ble_gatts_cb_param_t_gatts_disconnect_evt_param) -> Self {
        Self {
            id: param.conn_id,
            is_slave: param.link_role == 1,
            remote_bda: param.remote_bda,
        }
    }
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} ({}, slave: {})",
            self.remote_bda[0],
            self.remote_bda[1],
            self.remote_bda[2],
            self.remote_bda[3],
            self.remote_bda[4],
            self.remote_bda[5],
            self.id,
            self.is_slave,
        )
    }
}

impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.remote_bda.hash(state)
    }
}

impl std::cmp::PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.remote_bda == other.remote_bda
    }
}

impl std::cmp::Eq for Connection {}
