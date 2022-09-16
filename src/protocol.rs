use serde_repr::{Deserialize_repr, Serialize_repr};

use uuid::Uuid;
#[non_exhaustive]
pub struct AranetService;
#[non_exhaustive]
pub struct GenericService;
#[non_exhaustive]
pub struct CommonService;

impl AranetService {
    pub const UUID: Uuid = Uuid::from_u128(0xf0cd1400_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_CURRENT_READINGS: Uuid = Uuid::from_u128(0xf0cd1503_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_INTERVAL: Uuid = Uuid::from_u128(0xf0cd2002_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_SECONDS_SINCE_UPDATE: Uuid =
        Uuid::from_u128(0xf0cd2004_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_TOTAL_READINGS: Uuid = Uuid::from_u128(0xf0cd2001_95da_4f4b_9ac8_aa55d312af0c);
    pub const READ_HISTORY_READINGS: Uuid = Uuid::from_u128(0xf0cd2005_95da_4f4b_9ac8_aa55d312af0c);
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

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum LogParameter {
    Temperature = 1,
    Humidity = 2,
    Pressure = 3,
    Co2 = 4,
}

pub(crate) fn convert_temperature(temp: u16) -> f32 {
    (temp as f32 / 20.0) * 1.8 + 32_f32
}
pub(crate) fn convert_pressure(pressure: u16) -> f32 {
    pressure as f32 / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_parameter_serialize() {
        let x = LogParameter::Co2;
        let bin = bincode::serialize(&x).expect("value to serialize");
        assert_eq!(&bin, &[4u8]);
        let bin = &[3u8];
        let x: LogParameter = bincode::deserialize(bin).expect("value to deserialize");
        assert_eq!(x, LogParameter::Pressure);
    }
}
