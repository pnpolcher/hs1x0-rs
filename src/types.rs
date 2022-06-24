use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use serde::{Deserialize, Serialize};


#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct SystemGetSysInfoResponse {
    pub errcode: i64,
    pub sw_ver: String,
    pub hw_ver: String,
    #[serde(rename = "type")]
    pub hw_type: String,
    pub model: String,
    pub mac: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename= "hwId")]
    pub hw_id: String,
    #[serde(rename = "fwId")]
    pub fw_id: String,
    #[serde(rename = "oemId")]
    pub oem_id: String,
    pub alias: String,
    pub dev_name: String,
    pub icon_hash: String,
    pub relay_state: i64,
    pub on_time: i64,
    pub active_mode: String,
    pub feature: String,
    pub updating: i64,
    pub rssi: i64,
    pub led_off: i64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemResponse {
    pub get_sysinfo: SystemGetSysInfoResponse
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmeterGetRealtimeResponse {
    pub current: Option<f64>,
    pub current_ma: Option<f64>,
    pub voltage: Option<f64>,
    pub voltage_mv: Option<f64>,
    pub power: Option<f64>,
    pub power_mw: Option<f64>,
    pub total: Option<f64>,
    pub total_wh: Option<f64>,
    pub err_code: i64,
}

impl fmt::Display for EmeterGetRealtimeResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.voltage_mv.is_none() {
            write!(f, "V = {} V, I = {} A, P = {} W",
                   self.voltage.unwrap() / 1000.0,
                   self.current.unwrap() / 1000.0,
                   self.power.unwrap() / 1000.0
            )
        } else {
            write!(f, "V = {} V, I = {} A, P = {} W",
                   self.voltage_mv.unwrap() / 1000.0,
                   self.current_ma.unwrap() / 1000.0,
                   self.power_mw.unwrap() / 1000.0
            )
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmeterGetVGainIGainResponse {
    pub vgain: i64,
    pub igain: i64,
    pub err_code: i64,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmeterGetDaystatItem {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub energy: f64,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmeterGetDaystatResponse {
    pub day_list: Vec<EmeterGetDaystatItem>,
    pub err_code: i64,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmeterResponse {
    pub get_realtime: Option<EmeterGetRealtimeResponse>,
    pub get_vgain_igain: Option<EmeterGetVGainIGainResponse>,
    pub get_daystat: Option<EmeterGetDaystatResponse>
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlugResponse {
    pub system: Option<SystemResponse>,
    pub emeter: Option<EmeterResponse>
}

#[derive(Debug)]
pub struct PlugError {
    details: String
}

impl PlugError {
    pub fn new(msg: &str) -> PlugError {
        PlugError {
            details: msg.to_string()
        }
    }
}

impl fmt::Display for PlugError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for PlugError {
    fn description(&self) -> &str {
        &self.details
    }
}
