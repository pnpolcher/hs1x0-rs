pub mod types;

use chrono::{Date, Utc};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use serde_json::json;

use types::*;

/*
 * Protocol docs:
 *   https://github.com/softScheck/tplink-smartplug/blob/master/tplink-smarthome-commands.txt
 */

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

    pub fn get_realtime(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "emeter": {
                "get_realtime": {}
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn reboot(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "reboot": {
                    "delay": 1
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn reset_to_factory(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "reset": {
                    "delay": 1
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn turn_led_off(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_led_off": {
                    "off": 1
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_device_alias(&self, name: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_dev_alias": {
                    "alias": name
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_mac_address(&self, mac: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_mac_addr": {
                    "mac": mac
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_device_id(&self, device_id: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_device_id": {
                    "deviceId": device_id
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_hardware_id(&self, hardware_id: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_hw_id": {
                    "hwId": hardware_id
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_location(&self, latitude: f64, longitude: f64) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_dev_location": {
                    "longitude": longitude,
                    "latitude": latitude,
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn uboot_bootloader_check(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "test_check_uboot": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_device_icon(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "get_dev_icon": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_device_icon(&self, icon: &str, hash: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_dev_icon": {
                    "icon": icon,
                    "hash": hash,
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_test_mode(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "set_test_mode": {
                    "enable": 1
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn download_firmware_from_url(&self, url: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "download_firmware": {
                    "url": url
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_download_state(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "get_download_state": {}
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn flash_downloaded_firmware(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "flash_firmware": {}
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn check_config(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                "check_new_config": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn scan_available_aps(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "netif": {
                "get_scaninfo": {
                    "refresh": 1
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn connect_to_ap(&self, ssid: &str, password: &str)
        -> Result<PlugResponse, PlugError> {

        let v = json!({
            "netif": {
                "set_stainfo": {
                    "ssid": ssid,
                    "password": password,
                    "key_type": 3
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_cloud_info(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "cnCloud": {
                "get_info": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_firmware_list(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "cnCloud": {
                "get_intl_fw_list": {}
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_server_url(&self, server_url: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "cnCloud": {
                "set_server_url": {
                    "server": server_url,
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn connect_to_cloud(&self, user: &str, password: &str) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "cnCloud": {
                "bind": {
                    "username": user,
                    "password": password,
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn unregister_device(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "cnCloud": {
                "unbind": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_time(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "time": {
                "get_time": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_timezone(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "time": {
                "get_timezone": null
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn set_timezone(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "time": {
                "set_timezone": {
                    "year": 1,
                    "month": 2,
                    "mday": 3,
                    "hour": 4,
                    "min": 5,
                    "sec": 6,
                    "index": 42
                }
            }
        });

        send_command::<PlugResponse>(&self.ip, v.to_string())
    }

    pub fn get_meter_info(&self) -> Result<PlugResponse, PlugError> {
        let v = json!({
            "system": {
                 "get_sysinfo": {}
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
    use std::net::TcpStream;
    use std::time::Duration;
    use serde_json::json;
    use crate::{decrypt_payload, encrypt_payload, TpLinkDevice};

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
