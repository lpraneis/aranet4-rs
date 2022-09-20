use crate::sensor::protocol::LogParameter;
use chrono::Local;
use serde::{Deserialize, Serialize};

/// Size of history header in bytes
pub(crate) const HISTORY_HEADER_SIZE: usize = std::mem::size_of::<HistoryHeader>();

/// History reading header
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
    /// Decode from bytes
    pub(crate) fn decode(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
    pub(crate) fn get_data_start(&self) -> Option<chrono::DateTime<Local>> {
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
