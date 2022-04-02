use thiserror::Error;
#[derive(Debug, Error)]
pub enum SensorError {
    #[error("Bluetooth Error")]
    BluetoothError(#[from] btleplug::Error),
    #[error("Unable to create sensor")]
    CreationError,
    #[error("Unable to find sensor by address: {}", .0)]
    CannotFindAddress(String),
    #[error("Unable to find sensor by name")]
    CannotFindAddressByName,
    #[error("Cannot find characteristics")]
    CannotFindCharacteristics,
    #[error("Packet Deserialization Error")]
    ByteReadError(#[from] std::io::Error),
    #[error("Cannot parse bluetooth address: {}", .0)]
    BluetoothAddressParseError(#[from] btleplug::api::ParseBDAddrError),
}
