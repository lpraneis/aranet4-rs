use btleplug::api::{Characteristic, Peripheral, WriteType};
use chrono::Local;

use serde::{Deserialize, Serialize};

use crate::{
    convert_pressure, convert_temperature, AranetService, DataRecord, LogParameter, Sensor,
    SensorError,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct HistoryHeader {
    pub parameter: LogParameter,
    pub interval: u16,
    pub total_measurements: u16,
    pub time_since_last_measurement: u16,
    pub first_measure_index: u16,
    pub num_measurements: u8,
}

impl HistoryHeader {
    pub fn decode(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
    fn get_data_start(&self) -> Option<chrono::DateTime<Local>> {
        let beginning = chrono::Local::now();
        let time_since_last_measurement =
            chrono::Duration::seconds(self.time_since_last_measurement.into());
        let measurement_time = self.interval * self.num_measurements as u16;
        let measure_range = chrono::Duration::seconds(measurement_time.into());
        beginning
            .checked_sub_signed(time_since_last_measurement)?
            .checked_sub_signed(measure_range)
    }
}
pub(crate) const HISTORY_HEADER_SIZE: usize = std::mem::size_of::<HistoryHeader>();

#[derive(Debug, Serialize)]
pub(crate) struct HistoryRequest {
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

#[derive(Debug, Clone)]
pub struct HistoryInformation {
    interval: chrono::Duration,
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

impl Sensor {
    async fn get_temperature_history(
        &self,
        write_cmd: &Characteristic,
        reading_cmd: &Characteristic,
    ) -> Result<(HistoryInformation, Vec<f32>), SensorError> {
        let history_request = HistoryRequest {
            parameter: LogParameter::Temperature,
            first_index: 1,
        };
        self.aranet
            .write(
                write_cmd,
                &history_request.encode(),
                WriteType::WithoutResponse,
            )
            .await?;
        let mut ret = vec![];
        let mut header_data: Option<HistoryHeader> = None;
        while let Ok(bytes) = self.aranet.read(reading_cmd).await {
            if bytes.len() < HISTORY_HEADER_SIZE {
                return Err(SensorError::ProtocolError);
            }
            if let Some(header) = HistoryHeader::decode(&bytes[..HISTORY_HEADER_SIZE]) {
                // have we reached the end of the data stream?
                if header.num_measurements == 0 {
                    break;
                }

                header_data.get_or_insert(header);

                let end = std::mem::size_of::<u16>() * header.num_measurements as usize;
                let end = std::cmp::min(end, bytes.len());

                let vals: Vec<f32> = bytes[HISTORY_HEADER_SIZE..end]
                    .chunks_exact(2)
                    .into_iter()
                    .map(|x| u16::from_le_bytes([x[0], x[1]]))
                    .map(convert_temperature)
                    .collect();
                ret.extend(vals);
            } else {
                break;
            }
        }
        let header = header_data.ok_or(SensorError::ProtocolError)?;
        Ok((header.into(), ret))
    }
    async fn get_pressure_history(
        &self,
        write_cmd: &Characteristic,
        reading_cmd: &Characteristic,
    ) -> Result<Vec<f32>, SensorError> {
        let history_request = HistoryRequest {
            parameter: LogParameter::Pressure,
            first_index: 1,
        };
        self.aranet
            .write(
                write_cmd,
                &history_request.encode(),
                WriteType::WithoutResponse,
            )
            .await?;
        let mut ret = vec![];
        while let Ok(bytes) = self.aranet.read(reading_cmd).await {
            if bytes.len() < HISTORY_HEADER_SIZE {
                break;
            }
            if let Some(header) = HistoryHeader::decode(&bytes[..HISTORY_HEADER_SIZE]) {
                // have we reached the end of the data stream?
                if header.num_measurements == 0 {
                    break;
                }
                let end = std::mem::size_of::<u16>() * header.num_measurements as usize;
                let end = std::cmp::min(end, bytes.len());

                let vals: Vec<f32> = bytes[HISTORY_HEADER_SIZE..end]
                    .chunks_exact(2)
                    .into_iter()
                    .map(|x| u16::from_le_bytes([x[0], x[1]]))
                    .map(convert_pressure)
                    .collect();
                ret.extend(vals);
            } else {
                break;
            }
        }
        Ok(ret)
    }
    async fn get_humidity_history(
        &self,
        write_cmd: &Characteristic,
        reading_cmd: &Characteristic,
    ) -> Result<Vec<u8>, SensorError> {
        let history_request = HistoryRequest {
            parameter: LogParameter::Humidity,
            first_index: 1,
        };
        self.aranet
            .write(
                write_cmd,
                &history_request.encode(),
                WriteType::WithoutResponse,
            )
            .await?;
        let mut ret = vec![];
        while let Ok(bytes) = self.aranet.read(reading_cmd).await {
            if bytes.len() < HISTORY_HEADER_SIZE {
                break;
            }
            if let Some(header) = HistoryHeader::decode(&bytes[..HISTORY_HEADER_SIZE]) {
                // have we reached the end of the data stream?
                if header.num_measurements == 0 {
                    break;
                }
                let end = std::mem::size_of::<u8>() * header.num_measurements as usize;
                let end = std::cmp::min(end, bytes.len());

                ret.extend_from_slice(&bytes[HISTORY_HEADER_SIZE..end]);
            } else {
                break;
            }
        }
        Ok(ret)
    }
    async fn get_co2_history(
        &self,
        write_cmd: &Characteristic,
        reading_cmd: &Characteristic,
    ) -> Result<Vec<u16>, SensorError> {
        let history_request = HistoryRequest {
            parameter: LogParameter::Co2,
            first_index: 1,
        };
        self.aranet
            .write(
                write_cmd,
                &history_request.encode(),
                WriteType::WithoutResponse,
            )
            .await?;
        let mut ret = vec![];
        while let Ok(bytes) = self.aranet.read(reading_cmd).await {
            if bytes.len() < HISTORY_HEADER_SIZE {
                break;
            }
            if let Some(header) = HistoryHeader::decode(&bytes[..HISTORY_HEADER_SIZE]) {
                // have we reached the end of the data stream?
                if header.num_measurements == 0 {
                    break;
                }
                let end = std::mem::size_of::<u16>() * header.num_measurements as usize;
                let end = std::cmp::min(end, bytes.len());

                let vals: Vec<u16> = bytes[HISTORY_HEADER_SIZE..end]
                    .chunks_exact(2)
                    .into_iter()
                    .map(|x| u16::from_le_bytes([x[0], x[1]]))
                    .collect();
                ret.extend(vals);
            } else {
                break;
            }
        }
        Ok(ret)
    }

    /// Get the historical data for this sensor
    pub async fn get_historical_data(&self) -> Result<HistoryReadings, SensorError> {
        let write_cmd = self
            .get_characteristic(AranetService::WRITE_CMD)
            .ok_or(SensorError::CannotFindCharacteristics)?;
        let reading_cmd = self
            .get_characteristic(AranetService::READ_HISTORY_READINGS)
            .ok_or(SensorError::CannotFindCharacteristics)?;

        let (information, temperature) =
            self.get_temperature_history(write_cmd, reading_cmd).await?;
        let mut humidity = self.get_humidity_history(write_cmd, reading_cmd).await?;
        humidity.truncate(temperature.len());

        let mut co2 = self.get_co2_history(write_cmd, reading_cmd).await?;
        co2.truncate(temperature.len());

        let mut pressure = self.get_pressure_history(write_cmd, reading_cmd).await?;
        pressure.truncate(temperature.len());

        Ok(HistoryReadings {
            information,
            temperature,
            humidity,
            co2,
            pressure,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_header_serialize() {
        let x = HistoryHeader {
            parameter: LogParameter::Temperature,
            interval: 10,
            total_measurements: 20,
            time_since_last_measurement: 30,
            first_measure_index: 40,
            num_measurements: 50,
        };
        let bin = bincode::serialize(&x).expect("value to serialize");
        assert_eq!(bin, &[1u8, 10, 0, 20, 0, 30, 0, 40, 0, 50])
    }
    #[test]
    fn history_request_serialize() {
        let x = HistoryRequest {
            parameter: LogParameter::Temperature,
            first_index: 1,
        };
        let bin = bincode::serialize(&x).expect("value to serialize");
        assert_eq!(bin, &[1u8, 1, 0])
    }
}
