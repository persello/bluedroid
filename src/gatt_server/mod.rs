use std::sync::Mutex;

use esp_idf_sys::{
    esp_ble_addr_type_t_BLE_ADDR_TYPE_RPA_PUBLIC, esp_ble_adv_channel_t_ADV_CHNL_ALL,
    esp_ble_adv_data_t, esp_ble_adv_filter_t_ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
    esp_ble_adv_params_t, esp_ble_adv_type_t_ADV_TYPE_IND, esp_ble_gap_cb_param_t,
    esp_ble_gap_register_callback, esp_ble_gatts_cb_param_t, esp_ble_gatts_register_callback,
    esp_bluedroid_enable, esp_bluedroid_init, esp_bt_controller_config_t, esp_bt_controller_enable,
    esp_bt_controller_init, esp_bt_controller_mem_release, esp_bt_mode_t_ESP_BT_MODE_BLE,
    esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT, esp_gap_ble_cb_event_t, esp_gatt_if_t,
    esp_gatts_cb_event_t, esp_nofail, nvs_flash_erase, nvs_flash_init, AGC_RECORRECT_EN,
    BLE_HW_TARGET_CODE_ESP32C3_CHIP_ECO0, CFG_NASK, CONFIG_BT_CTRL_ADV_DUP_FILT_MAX,
    CONFIG_BT_CTRL_BLE_MAX_ACT_EFF, CONFIG_BT_CTRL_BLE_STATIC_ACL_TX_BUF_NB,
    CONFIG_BT_CTRL_CE_LENGTH_TYPE_EFF, CONFIG_BT_CTRL_COEX_PHY_CODED_TX_RX_TLIM_EFF,
    CONFIG_BT_CTRL_DFT_TX_POWER_LEVEL_EFF, CONFIG_BT_CTRL_HCI_TL_EFF, CONFIG_BT_CTRL_HW_CCA_EFF,
    CONFIG_BT_CTRL_HW_CCA_VAL, CONFIG_BT_CTRL_MODE_EFF, CONFIG_BT_CTRL_PINNED_TO_CORE,
    CONFIG_BT_CTRL_RX_ANTENNA_INDEX_EFF, CONFIG_BT_CTRL_SLEEP_CLOCK_EFF,
    CONFIG_BT_CTRL_SLEEP_MODE_EFF, CONFIG_BT_CTRL_TX_ANTENNA_INDEX_EFF,
    ESP_BLE_ADV_FLAG_BREDR_NOT_SPT, ESP_BLE_ADV_FLAG_GEN_DISC, ESP_BT_CTRL_CONFIG_MAGIC_VAL,
    ESP_BT_CTRL_CONFIG_VERSION, ESP_ERR_NVS_NEW_VERSION_FOUND, ESP_ERR_NVS_NO_FREE_PAGES,
    ESP_TASK_BT_CONTROLLER_PRIO, ESP_TASK_BT_CONTROLLER_STACK, MESH_DUPLICATE_SCAN_CACHE_SIZE,
    NORMAL_SCAN_DUPLICATE_CACHE_SIZE, SCAN_DUPLICATE_MODE, SCAN_DUPLICATE_TYPE_VALUE,
    SLAVE_CE_LEN_MIN_DEFAULT,
};
use lazy_static::lazy_static;
use log::{info, warn};

use crate::{leaky_box_raw, utilities::Appearance};

pub use characteristic::Characteristic;
pub use descriptor::Descriptor;
pub use profile::Profile;
pub use service::Service;

// Structs.
mod characteristic;
mod descriptor;
mod profile;
mod service;

// Custom stuff.
mod custom_attributes;

// Event handler.
mod gap_event_handler;
mod gatts_event_handler;

lazy_static! {
    pub static ref GLOBAL_GATT_SERVER: Mutex<Option<GattServer>> = Mutex::new(Some(GattServer {
        profiles: Vec::new(),
        started: false,
        advertisement_parameters: esp_ble_adv_params_t {
            adv_int_min: 0x20,
            adv_int_max: 0x40,
            adv_type: esp_ble_adv_type_t_ADV_TYPE_IND,
            own_addr_type: esp_ble_addr_type_t_BLE_ADDR_TYPE_RPA_PUBLIC,
            channel_map: esp_ble_adv_channel_t_ADV_CHNL_ALL,
            adv_filter_policy: esp_ble_adv_filter_t_ADV_FILTER_ALLOW_SCAN_ANY_CON_ANY,
            ..Default::default()
        },
        advertisement_data: esp_ble_adv_data_t {
            set_scan_rsp: false,
            include_name: true,
            include_txpower: true,
            min_interval: 0x0006,
            max_interval: 0x0010,
            appearance: Appearance::GenericUnknown.into(),
            manufacturer_len: 0,
            p_manufacturer_data: std::ptr::null_mut(),
            service_data_len: 0,
            p_service_data: std::ptr::null_mut(),
            service_uuid_len: 0,
            p_service_uuid: std::ptr::null_mut(),
            flag: (ESP_BLE_ADV_FLAG_GEN_DISC | ESP_BLE_ADV_FLAG_BREDR_NOT_SPT) as u8,
        },
        scan_response_data: esp_ble_adv_data_t {
            set_scan_rsp: true,
            include_name: false,
            include_txpower: false,
            min_interval: 0x0006,
            max_interval: 0x0010,
            appearance: Appearance::GenericUnknown.into(),
            manufacturer_len: 0,
            p_manufacturer_data: std::ptr::null_mut(),
            service_data_len: 0,
            p_service_data: std::ptr::null_mut(),
            service_uuid_len: 0,
            p_service_uuid: std::ptr::null_mut(),
            flag: (ESP_BLE_ADV_FLAG_GEN_DISC | ESP_BLE_ADV_FLAG_BREDR_NOT_SPT) as u8,
        },
        advertisement_configured: false,
        device_name: "ESP32".to_string(),
    }));
}

pub struct GattServer {
    profiles: Vec<Profile>,
    started: bool,
    advertisement_parameters: esp_ble_adv_params_t,
    advertisement_data: esp_ble_adv_data_t,
    scan_response_data: esp_ble_adv_data_t,
    device_name: String,
    advertisement_configured: bool,
}

unsafe impl Send for GattServer {}

impl GattServer {
    pub fn start(&mut self) {
        if self.started {
            warn!("GATT server already started.");
            return;
        }

        self.started = true;
        self.initialise_ble_stack();

        // Registration of profiles, services, characteristics and descriptors.
        self.profiles.iter().for_each(|profile: &Profile| {
            profile.register_self();
        })
    }

    pub fn device_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        if self.advertisement_configured {
            warn!(
                "Device name already set. Please set the device name before starting the server."
            );
            return self;
        }

        self.device_name = name.into();
        self.device_name.push('\0');

        self
    }

    pub fn appearance(&mut self, appearance: Appearance) -> &mut Self {
        if self.advertisement_configured {
            warn!("Appearance already set. Please set the appearance before starting the server.");
            return self;
        }
        
        self.advertisement_data.appearance = appearance.into();
        self.scan_response_data.appearance = appearance.into();

        self
    }

    pub fn add_profiles(&mut self, profiles: &[Profile]) -> &mut Self {
        self.profiles.append(&mut profiles.to_vec());
        if self.started {
            warn!("In order to register the newly added profiles, you'll need to restart the GATT server.");
        }

        self
    }

    pub fn set_adv_params(&mut self, params: esp_ble_adv_params_t) -> &mut Self {
        self.advertisement_parameters = params;
        self
    }

    pub fn set_adv_data(&mut self, data: esp_ble_adv_data_t) -> &mut Self {
        self.advertisement_data = data;

        self
    }

    pub fn advertise_service(&mut self, service: Service) -> &mut Self {
        self.scan_response_data.p_service_uuid =
            leaky_box_raw!(service.uuid.as_uuid128_array()) as *mut u8;
        self.scan_response_data.service_uuid_len = service.uuid.as_uuid128_array().len() as u16;

        self
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
            esp_nofail!(esp_ble_gatts_register_callback(Some(
                Self::default_gatts_callback
            )));
            esp_nofail!(esp_ble_gap_register_callback(Some(
                Self::default_gap_callback
            )));
        }
    }

    /// Calls the global server's GATT event callback.
    ///
    /// This is a bad workaround, and only works because we have a singleton server.
    extern "C" fn default_gatts_callback(
        event: esp_gatts_cb_event_t,
        gatts_if: esp_gatt_if_t,
        param: *mut esp_ble_gatts_cb_param_t,
    ) {
        GLOBAL_GATT_SERVER
            .lock()
            .expect("Cannot lock global GATT server.")
            .as_mut()
            .expect("Cannot get mutable reference to global GATT server.")
            .gatts_event_handler(event, gatts_if, param);
    }

    /// Calls the global server's GAP event callback.
    ///
    /// This is a bad workaround, and only works because we have a singleton server.
    extern "C" fn default_gap_callback(
        event: esp_gap_ble_cb_event_t,
        param: *mut esp_ble_gap_cb_param_t,
    ) {
        GLOBAL_GATT_SERVER
            .lock()
            .expect("Cannot lock global GATT server.")
            .as_mut()
            .expect("Cannot get mutable reference to global GATT server.")
            .gap_event_handler(event, param);
    }
}
