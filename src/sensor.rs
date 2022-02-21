use btleplug::api::{BDAddr, Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeSet;
use std::fmt;
use std::io::Cursor;
use std::time::Duration;
use tokio::time;

pub use crate::protocol::*;

#[derive(Clone)]
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
    /// construct an empty set of readings. Can be used for displays, etc.
    pub fn empty() -> SensorReadings {
        SensorReadings {
            co2_level: 0,
            temperature: 0,
            pressure: 0,
            humidity: 0,
            battery: 0,
            status_color: 0,
        }
    }
    /// construct a `SensorReadings` from a raw bytestream retrieved from the sensor
    fn from_raw(bytes: Vec<u8>) -> Option<SensorReadings> {
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
    /// CO2 level, expressed in ppm
    pub fn co2_level(&self) -> u16 {
        self.co2_level
    }
    /// Temperature in Fahrenheit
    pub fn temperature(&self) -> f32 {
        (self.temperature as f32 * 1.8) + 32_f32
    }
    /// Pressure in kpa
    pub fn pressure(&self) -> u16 {
        self.pressure
    }
    /// Humidity in percent humidity
    pub fn humidity(&self) -> u8 {
        self.humidity
    }
    /// Battery percent
    pub fn battery(&self) -> u8 {
        self.battery
    }
}
pub struct Sensor {
    aranet: Peripheral,
    characteristics: BTreeSet<Characteristic>,
}

impl Sensor {
    async fn find_sensor_by_name(central: &Adapter) -> Option<Peripheral> {
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
    async fn find_sensor_by_addr(central: &Adapter, addr: &str) -> Option<Peripheral> {
        let bdaddr = BDAddr::from_str_delim(addr).expect("Cannot parse Bluetooth Address");
        for p in central.peripherals().await.unwrap() {
            if p.properties().await.unwrap().unwrap().address.eq(&bdaddr) {
                return Some(p);
            }
        }
        None
    }
    async fn init(central: &Adapter, addr: Option<String>) -> Option<Sensor> {
        let aranet = match addr {
            Some(addr) => Sensor::find_sensor_by_addr(central, &addr)
                .await
                .expect("No sensor found"),
            None => Sensor::find_sensor_by_name(central)
                .await
                .expect("No sensor found"),
        };

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

pub struct SensorManager {}
impl SensorManager {
    pub async fn init(addr: Option<String>) -> Option<Sensor> {
        let manager = Manager::new().await.unwrap();

        // get the first bluetooth adapter
        let central = manager
            .adapters()
            .await
            .expect("Unable to fetch adapter list.")
            .into_iter()
            .next()
            .expect("Unable to find adapters.");

        central
            .start_scan(ScanFilter::default())
            .await
            .expect("Cannot scan for devices");
        time::sleep(Duration::from_secs(2)).await;

        let sensor = Sensor::init(&central, addr)
            .await
            .expect("Could not create sensor");
        Some(sensor)
    }
}
