use super::{header::HistoryHeader, record::DataRecord};
use crate::sensor::protocol::LogParameter;
use chrono::Local;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct HistoryRequest {
    pub parameter: LogParameter,
    pub first_index: u16,
}

impl HistoryRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![0x61];
        data.extend(bincode::serialize(self).unwrap());
        data
    }
}

/// Metadata about a [`HistoryReadings`]
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct HistoryInformation {
    pub interval: chrono::Duration,
    beginning: chrono::DateTime<Local>,
}
impl From<HistoryHeader> for HistoryInformation {
    fn from(header: HistoryHeader) -> Self {
        let interval = chrono::Duration::seconds(header.interval.into());
        let beginning = header.get_data_start().unwrap_or_else(chrono::Local::now);
        Self {
            interval,
            beginning,
        }
    }
}

/// Historical Readings from Sensor
#[derive(Debug, Clone)]
pub struct HistoryReadings {
    pub information: HistoryInformation,
    pub temperature: Vec<f32>,
    pub humidity: Vec<u8>,
    pub co2: Vec<u16>,
    pub pressure: Vec<f32>,
}

impl HistoryReadings {
    /// Get a view of the data as a vector of [`DataRecord`]
    pub fn as_records(&self) -> Vec<DataRecord> {
        self.temperature
            .iter()
            .zip(self.humidity.iter())
            .zip(self.co2.iter())
            .zip(self.pressure.iter())
            .map(|tup| {
                let (((temperature, humidity), co2), pressure) = tup;
                DataRecord {
                    temperature: *temperature,
                    humidity: *humidity,
                    pressure: *pressure,
                    co2: *co2,
                }
            })
            .collect()
    }
}
