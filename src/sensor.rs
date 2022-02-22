use btleplug::api::{BDAddr, Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeSet;
use std::io::Cursor;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use crate::data::SensorReadings;
pub use crate::protocol::*;

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
    fn get_characteristic(&self, uuid: Uuid) -> Option<&Characteristic> {
        self.characteristics.iter().find(|c| c.uuid == uuid)
    }
    pub async fn read_current_values(&self) -> Option<SensorReadings> {
        let cmd_chars = self
            .get_characteristic(AranetService::READ_CURRENT_READINGS)
            .expect("Unable to find characteristics");
        let vals = self
            .aranet
            .read(cmd_chars)
            .await
            .expect("Cannot read current values");
        SensorReadings::from_raw(vals)
    }
    pub async fn last_update_time(&self) -> Duration {
        let cmd_chars = self
            .get_characteristic(AranetService::READ_SECONDS_SINCE_UPDATE)
            .expect("Unable to find characteristics");
        let bytes = self
            .aranet
            .read(cmd_chars)
            .await
            .expect("Cannot read seconds since update");
        let mut reader = Cursor::new(bytes);
        let seconds_ago = reader.read_u16::<LittleEndian>().unwrap();
        Duration::from_secs(seconds_ago.into())
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
