use btleplug::api::{
    BDAddr, Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::stream::StreamExt;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use crate::{protocol::*, DataRecord, SensorError, SensorReadings};

pub struct Sensor {
    aranet: Peripheral,
    characteristics: BTreeSet<Characteristic>,
}

impl Sensor {
    async fn find_sensor_by_name(central: &Adapter) -> Result<Peripheral, SensorError> {
        for p in central.peripherals().await? {
            if let Some(peripheral) = p.properties().await? {
                if peripheral
                    .local_name
                    .iter()
                    .any(|name| name.contains("Aranet4"))
                {
                    return Ok(p);
                }
            }
        }
        Err(SensorError::CannotFindAddressByName)
    }
    async fn find_sensor_by_addr(central: &Adapter, addr: &str) -> Result<Peripheral, SensorError> {
        let bdaddr = BDAddr::from_str_delim(addr)?;
        for p in central.peripherals().await? {
            if let Some(peripheral) = p.properties().await? {
                if peripheral.address.eq(&bdaddr) {
                    return Ok(p);
                }
            }
        }
        Err(SensorError::CannotFindAddress(addr.to_string()))
    }
    async fn init(central: &Adapter, addr: Option<String>) -> Result<Sensor, SensorError> {
        let aranet = if let Some(address) = addr {
            Sensor::find_sensor_by_addr(central, &address).await?
        } else {
            Sensor::find_sensor_by_name(central).await?
        };

        aranet.connect().await?;
        aranet.discover_services().await?;
        let chars = aranet.characteristics();

        Ok(Sensor {
            aranet,
            characteristics: chars,
        })
    }
    fn get_characteristic(&self, uuid: Uuid) -> Option<&Characteristic> {
        self.characteristics.iter().find(|c| c.uuid == uuid)
    }
    pub async fn read_current_values(&self) -> Result<SensorReadings, SensorError> {
        if let Some(cmd_chars) = self.get_characteristic(AranetService::READ_CURRENT_READINGS) {
            let vals = self.aranet.read(cmd_chars).await?;
            SensorReadings::from_raw(vals)
        } else {
            Err(SensorError::CannotFindCharacteristics)
        }
    }
    pub async fn last_update_time(&self) -> Result<Duration, SensorError> {
        if let Some(cmd_chars) = self.get_characteristic(AranetService::READ_SECONDS_SINCE_UPDATE) {
            let bytes = self.aranet.read(cmd_chars).await?;
            let mut reader = Cursor::new(bytes);
            let seconds_ago = reader.read_u16::<LittleEndian>()?;
            Ok(Duration::from_secs(seconds_ago.into()))
        } else {
            Err(SensorError::CannotFindCharacteristics)
        }
    }
    pub async fn get_historical_data(&self) -> Result<Vec<DataRecord>, SensorError> {
        let cmd = self.get_characteristic(AranetService::WRITE_CMD);
        let readings = self.get_characteristic(AranetService::READ_HISTORY_READINGS);
        match (cmd, readings) {
            (Some(write_cmd), Some(readings_char)) => {
                self.aranet
                    .write(
                        write_cmd,
                        &[0x82, 0x01, 0x00, 0x00, 0xde, 0x01, 0x3d, 0x04],
                        WriteType::WithoutResponse,
                    )
                    .await?;
                self.aranet.subscribe(readings_char).await?;
                let mut notif_stream = self.aranet.notifications().await?;
                while let Some(s) = notif_stream.next().await {
                    println!("{:x?}", s.value);
                }

                Ok(Vec::new())
            }
            (_, _) => Err(SensorError::CannotFindCharacteristics),
        }
    }
}

pub struct SensorManager {}
impl SensorManager {
    pub async fn init(addr: Option<String>) -> Result<Sensor, SensorError> {
        let manager = Manager::new().await?;

        // get the first bluetooth adapter
        if let Some(central) = manager.adapters().await?.into_iter().next() {
            central.start_scan(ScanFilter::default()).await?;
            time::sleep(Duration::from_secs(2)).await;

            let sensor = Sensor::init(&central, addr).await?;
            Ok(sensor)
        } else {
            Err(SensorError::CreationError)
        }
    }
}
