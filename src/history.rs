use self::{
    header::{HistoryHeader, HISTORY_HEADER_SIZE},
    readings::{HistoryInformation, HistoryReadings, HistoryRequest},
};
use crate::{
    error::SensorError,
    sensor::{
        protocol::{convert_pressure, convert_temperature, AranetService, LogParameter},
        Sensor,
    },
};
use btleplug::api::{Characteristic, Peripheral, WriteType};
mod header;
pub mod readings;
pub mod record;

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
