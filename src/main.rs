#![allow(dead_code)]
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::io::Cursor;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

#[non_exhaustive]
struct AranetService;
#[non_exhaustive]
struct GenericService;
#[non_exhaustive]
struct CommonService;
#[non_exhaustive]
struct LogParameter;

impl AranetService {
    pub const UUID: Uuid = Uuid::from_u128(0xf0cd1400_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_CURRENT_READINGS: Uuid = Uuid::from_u128(0xf0cd1503_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_INTERVAL: Uuid = Uuid::from_u128(0xf0cd2002_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_SECONDS_SINCE_UPDATE: Uuid =
        Uuid::from_u128(0xf0cd2004_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_TOTAL_READINGS: Uuid = Uuid::from_u128(0xf0cd2001_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_HISTORY_READINGS: Uuid = Uuid::from_u128(0xf0cd2003_95da_4f4b_9ac8_aa55d312af0c);
    pub const WRITE_CMD: Uuid = Uuid::from_u128(0xf0cd1402_95da_4f4b_9ac8_aa55d312af0c);
}

impl GenericService {
    pub const UUID: Uuid = Uuid::from_u128(0x00001800_0000_1000_8000_00805f9b34fb);
    pub const READ_DEVICE_NAME: Uuid = Uuid::from_u128(0x00002a00_0000_1000_8000_00805f9b34fb);
}

impl CommonService {
    pub const UUID: Uuid = Uuid::from_u128(0x0000180a_0000_1000_8000_00805f9b34fb);
    pub const READ_MANUFACTURER_NAME: Uuid =
        Uuid::from_u128(0x00002a29_0000_1000_8000_00805f9b34fb);
    pub const READ_MODEL_NUMBER: Uuid = Uuid::from_u128(0x00002a24_0000_1000_8000_00805f9b34fb);
    pub const READ_SERIAL_NO: Uuid = Uuid::from_u128(0x00002a25_0000_1000_8000_00805f9b34fb);
    pub const READ_HW_REV: Uuid = Uuid::from_u128(0x00002a27_0000_1000_8000_00805f9b34fb);
    pub const READ_SW_REV: Uuid = Uuid::from_u128(0x00002a28_0000_1000_8000_00805f9b34fb);
    pub const READ_BATTERY: Uuid = Uuid::from_u128(0x00002a19_0000_1000_8000_00805f9b34fb);
}

impl LogParameter {
    pub const TEMPERATURE: i32 = 1;
    pub const HUMIDITY: i32 = 2;
    pub const PRESSURE: i32 = 3;
    pub const CO2: i32 = 4;
}

struct Sensor {
    aranet: Peripheral,
    characteristics: BTreeSet<Characteristic>,
}

struct SensorReadings {
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

impl Sensor {
    pub async fn find_sensor(central: &Adapter) -> Option<Peripheral> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let sensor = Sensor::init(&central)
        .await
        .expect("Could not create sensor");
    let readings = sensor
        .read_current_values()
        .await
        .expect("Could not read current values");
    println!("{}", readings);
    Ok(())
}
