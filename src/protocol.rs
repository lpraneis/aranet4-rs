use uuid::Uuid;
#[non_exhaustive]
pub struct AranetService;
#[non_exhaustive]
pub struct GenericService;
#[non_exhaustive]
pub struct CommonService;
#[non_exhaustive]
pub struct LogParameter;

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
