use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub enum DeviceType {
    Plug,
    Bulb,
    Strip,
    Unknown,
}

fn size_to_bytes(size: u32) -> [u8;4] {
    let b1 = ((size >> 24) & 0xff) as u8;
    let b2 = ((size >> 16) & 0xff) as u8;
    let b3 = ((size >> 8) & 0xff) as u8;
    let b4 = (size & 0xff) as u8;

    return [b1, b2, b3, b4];
}

fn size_from_bytes(size: &[u8]) -> usize {
    return ((size[0] as usize) << 24) |
        ((size[1] as usize) << 16) |
        ((size[2] as usize) << 8) |
        size[3] as usize;
}

fn encrypt_payload(data: Vec<u8>) -> Vec<u8> {
    let it = data.iter();
    let mut v2 = Vec::new();
    let mut key = 171;

    size_to_bytes(data.len() as u32).map(|x| v2.push(x));

    for b in it {
        let tmp = *b ^ key;
        v2.push(tmp);
        key = tmp;
    }

    v2
}

fn decrypt_payload(data: &[u8]) -> Vec<u8> {

    let payload_size = size_from_bytes(&data[0..4]);
    let mut v2 = Vec::new();
    let mut key = 171u8;

    for idx in 4..payload_size+4 {
        let tmp = data[idx] ^ key;
        v2.push(tmp);
        key = data[idx];
    }

    v2
}

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
    fn new(msg: &str) -> PlugError {
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

pub struct TpLinkDevice {
    ip: String
}

fn send_command<T>(ip: &str, s: String) -> Result<T, PlugError>
where
    T: serde::de::DeserializeOwned
{
    match TcpStream::connect(ip) {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();

            let payload = encrypt_payload(s.as_bytes().to_vec());
            match stream.write(payload.as_slice()) {
                Ok(_v) => 0,
                Err(e) => return Err(PlugError::new("Write failed"))
            };

            let mut buf = [0u8; 2048];
            let size = match stream.read(&mut buf) {
                Ok(v) => v,
                Err(e) => return Err(PlugError::new("Read failed"))
            };

            let decrypted = match String::from_utf8(decrypt_payload(&buf[0..size])) {
                Ok(v) => v,
                Err(e) => return Err(PlugError::new("Decoding failed"))
            };

            match serde_json::from_str(decrypted.as_str()) {
                Ok(result) => Ok(result),
                Err(e) => return Err(PlugError::new(
                    format!("Deserialization failed. Reason: {}", e.to_string()).as_str()))
            }
        }
        Err(_) => Err(PlugError::new("Connection error")),
    }
}

impl TpLinkDevice {
    pub fn new(ip: &'static str) -> TpLinkDevice {
        TpLinkDevice {
            ip: String::from(ip)
        }
    }

    fn set_relay_state(&self, state: u8) -> Result<PlugResponse, PlugError> {
        let cmd = json!({
            "system": {
                "set_relay_state": {
                    "state": state
                }
            }
        });
        send_command(&self.ip, cmd.to_string())
    }

    pub fn on(&self) -> Result<PlugResponse, PlugError> {
        self.set_relay_state(1)
    }

    pub fn off(&self) -> Result<PlugResponse, PlugError> {
        self.set_relay_state(0)
    }

    pub fn get_realtime(&self) -> Result<PlugResponse, PlugError>{
        let v = json!({
            "emeter": {
                "get_realtime": {}
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_realtime_current_voltage() -> (f32, f32) {
        let cmd = json!({
            "emeter": {
                "get_realtime": {}
            }
        });
        (1 as f32, 1 as f32)
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpStream};
    use std::time::Duration;
    use crate::{decrypt_payload, encrypt_payload, TpLinkDevice};
    use serde_json::json;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_encrypt_payload() {
        let ep = encrypt_payload(
            String::from("{\"system\":{\"set_relay_state\":{\"state\":0}}}").as_bytes().to_vec());
        let dp = decrypt_payload(ep.as_slice());
        // TODO: test input and output strings are equal.
    }

    #[test]
    fn test_get_realtime() {
        let device = TpLinkDevice::new("192.168.1.115:9999");
        match device.get_realtime() {
            Ok(result) => { println!("{}", result.emeter.unwrap().get_realtime.unwrap()) },
            Err(e) => { eprintln!("{}", e) }
        }
    }

    #[test]
    fn test_comm() {
        let v = json!({
            "emeter": {
                "get_realtime": {}
            }
        });

        let ev = encrypt_payload(v.to_string().as_bytes().to_vec());

        match TcpStream::connect("192.168.1.115:9999") {
            Ok(mut stream) => {
                println!("{}", v.to_string());
                let size = stream.write(ev.as_slice()).unwrap();
                println!("{:?}", ev.as_slice());
                println!("Size = {}", size);
                let mut buf = [0u8; 2048];
                stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
                let size = stream.read(&mut buf).unwrap();
                println!("Size = {}", size);
                println!("Response = {}", String::from_utf8(
                    decrypt_payload(&buf[0..size])).unwrap());

                // Ok(String::from_utf8(buf[0..size].to_vec()).unwrap())
            }
            Err(_) => (), //Err(String::from("Failed connecting")),
        }
    }
}
