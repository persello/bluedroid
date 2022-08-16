use std::ptr::replace;
use std::sync::Mutex;

use esp_idf_sys::*;
use lazy_static::lazy_static;
use log::{info, warn};

pub use application::Application;
pub use characteristic::Characteristic;
pub use descriptor::Descriptor;
pub use service::Service;

use crate::leaky_box_raw;

// Structs.
mod application;
mod characteristic;
mod descriptor;
mod service;

// Event handler.
mod gap_event_handler;
mod gatts_event_handler;

lazy_static! {
    static ref GLOBAL_GATT_SERVER: Mutex<Option<GattServer>> = Mutex::new(Some(GattServer {
        applications: Vec::new(),
        started: false,
    }));
}

pub struct GattServer {
    applications: Vec<Application>,
    started: bool,
}

impl GattServer {
    pub fn take() -> Option<Self> {
        if let Ok(mut server) = GLOBAL_GATT_SERVER.try_lock() {
            let mut server = server.take();
            unsafe { replace(&mut server, None) }
        } else {
            None
        }
    }

    pub fn start(&mut self) {
        if self.started {
            warn!("GATT server already started.");
            return;
        }

        self.started = true;
        self.initialise_ble_stack();

        // Registration of applications, services, characteristics and descriptors.
        self.applications.iter().for_each(|application| {
            application.register_self();
        })
    }

    pub fn add_applications(&mut self, applications: &[Application]) {
        self.applications.append(&mut applications.to_vec());
        if self.started {
            warn!("In order to register the newly added applications, you'll need to restart the GATT server.");
        }
    }

    fn initialise_ble_stack(&mut self) {
        info!("Initialising BLE stack.");

        // NVS initialisation.
        unsafe {
            let result = nvs_flash_init();
            if result == ESP_ERR_NVS_NO_FREE_PAGES || result == ESP_ERR_NVS_NEW_VERSION_FOUND {
                warn!("NVS initialisation failed. Erasing NVS.");
                esp_nofail!(nvs_flash_erase());
                esp_nofail!(nvs_flash_init());
            }
        }

        let default_controller_configuration = esp_bt_controller_config_t {
            magic: ESP_BT_CTRL_CONFIG_MAGIC_VAL,
            version: ESP_BT_CTRL_CONFIG_VERSION,
            controller_task_stack_size: ESP_TASK_BT_CONTROLLER_STACK as u16,
            controller_task_prio: ESP_TASK_BT_CONTROLLER_PRIO as u8,
            controller_task_run_cpu: CONFIG_BT_CTRL_PINNED_TO_CORE as u8,
            bluetooth_mode: CONFIG_BT_CTRL_MODE_EFF as u8,
            ble_max_act: CONFIG_BT_CTRL_BLE_MAX_ACT_EFF as u8,
            sleep_mode: CONFIG_BT_CTRL_SLEEP_MODE_EFF as u8,
            sleep_clock: CONFIG_BT_CTRL_SLEEP_CLOCK_EFF as u8,
            ble_st_acl_tx_buf_nb: CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB as u8,
            ble_hw_cca_check: CONFIG_BT_CTRL_HW_CCA_EFF as u8,
            ble_adv_dup_filt_max: CONFIG_BT_CTRL_ADV_DUP_FILT_MAX as u16,
            coex_param_en: false,
            ce_len_type: CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF as u8,
            coex_use_hooks: false,
            hci_tl_type: CONFIG_BT_CTRL_HCI_TL_EFF as u8,
            hci_tl_funcs: std::ptr::null_mut(),
            txant_dft: CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF as u8,
            rxant_dft: CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF as u8,
            txpwr_dft: CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF as u8,
            cfg_mask: CFG_NASK,
            scan_duplicate_mode: SCAN_DUPLICATE_MODE as u8,
            scan_duplicate_type: SCAN_DUPLICATE_TYPE_VALUE as u8,
            normal_adv_size: NORMAL_SCAN_DUPLICATE_CACHE_SIZE as u16,
            mesh_adv_size: MESH_DUPLICATE_SCAN_CACHE_SIZE as u16,
            coex_phy_coded_tx_rx_time_limit: CONFIG_BT_CTRL_COEX_PHY_CODED_TX_RX_TLIM_EFF as u8,
            hw_target_code: BLE_HW_TARGET_CODE_ESP32C3_CHIP_ECO0,
            slave_ce_len_min: SLAVE_CE_LEN_MIN_DEFAULT as u8,
            hw_recorrect_en: AGC_RECORRECT_EN as u8,
            cca_thresh: CONFIG_BT_CTRL_HW_CCA_VAL as u8,
        };

        // BLE controller initialisation.
        unsafe {
            esp_nofail!(esp_bt_controller_mem_release(
                esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT
            ));
            esp_nofail!(esp_bt_controller_init(leaky_box_raw!(
                default_controller_configuration
            )));
            esp_nofail!(esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE));
            esp_nofail!(esp_bluedroid_init());
            esp_nofail!(esp_bluedroid_enable());
            // esp_nofail!(esp_ble_gatts_register_callback(Some()));
            // esp_nofail!(esp_ble_gap_register_callback(Some()));
        }
    }
}
