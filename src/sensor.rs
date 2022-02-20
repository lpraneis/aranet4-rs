use btleplug::api::{Central, Characteristic, Peripheral as _};
use btleplug::platform::{Adapter, Peripheral};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeSet;
use std::fmt;
use std::io::Cursor;

pub use crate::protocol::*;

pub struct SensorReadings {
    co2_level: u16,
    temperature: u16,
    pressure: u16,
    humidity: u8,
    battery: u8,
    status_color: u8,
}
impl fmt::Display for SensorReadings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CO2: {}ppm, Temperature: {}F, Pressure : {}kpa, Humidity : {}, Battery: {}, Status Color: {}",
            self.co2_level, self.temperature, self.pressure, self.humidity, self.battery, self.status_color
        )
    }
}

impl SensorReadings {
    pub fn from_raw(bytes: Vec<u8>) -> Option<SensorReadings> {
        let mut reader = Cursor::new(bytes);
        let co2_level = reader.read_u16::<LittleEndian>().unwrap();
        let temperature = reader.read_u16::<LittleEndian>().unwrap();
        let pressure = reader.read_u16::<LittleEndian>().unwrap();
        let humidity = reader.read_u8().unwrap();
        let battery = reader.read_u8().unwrap();
        let status_color = reader.read_u8().unwrap();

        Some(SensorReadings {
            co2_level,
            temperature: temperature / 20,
            pressure: pressure / 10,
            humidity,
            battery,
            status_color,
        })
    }
}
pub struct Sensor {
    aranet: Peripheral,
    characteristics: BTreeSet<Characteristic>,
}

impl Sensor {
    async fn find_sensor(central: &Adapter) -> Option<Peripheral> {
        for p in central.peripherals().await.unwrap() {
            if p.properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| name.contains("Aranet4"))
            {
                return Some(p);
            }
        }
        None
    }
    pub async fn init(central: &Adapter) -> Option<Sensor> {
        let aranet = Sensor::find_sensor(central).await.expect("No lights found");

        aranet.connect().await.expect("Could not connect");
        aranet
            .discover_services()
            .await
            .expect("Could not discover services");
        let chars = aranet.characteristics();

        Some(Sensor {
            aranet,
            characteristics: chars,
        })
    }
    pub async fn read_current_values(&self) -> Option<SensorReadings> {
        let cmd_chars = self
            .characteristics
            .iter()
            .find(|c| c.uuid == AranetService::READ_CURRENT_READINGS)
            .expect("Unable to find characteristics");
        let vals = self
            .aranet
            .read(cmd_chars)
            .await
            .expect("Cannot read current values");
        SensorReadings::from_raw(vals)
    }
}
