use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::Cursor;

use crate::{
    error::SensorError,
    sensor::protocol::{convert_pressure, convert_temperature},
};

/// One-time readings from sensor
#[derive(Clone, Default)]
pub struct SensorReadings {
    /// CO2 level, expressed in ppm
    pub co2_level: u16,
    /// Temperature in Fahrenheit
    pub temperature: f32,
    /// Pressure in kpa
    pub pressure: f32,
    /// Humidity in percent humidity
    pub humidity: u8,
    /// Battery percent
    pub battery: u8,
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
    /// construct a `SensorReadings` from a raw bytestream retrieved from the sensor
    pub(crate) fn from_raw(bytes: Vec<u8>) -> Result<SensorReadings, SensorError> {
        let mut reader = Cursor::new(bytes);
        let co2_level = reader.read_u16::<LittleEndian>()?;
        let temperature = reader.read_u16::<LittleEndian>()?;
        let pressure = reader.read_u16::<LittleEndian>()?;
        let humidity = reader.read_u8()?;
        let battery = reader.read_u8()?;
        let status_color = reader.read_u8()?;

        Ok(SensorReadings {
            co2_level,
            temperature: convert_temperature(temperature),
            pressure: convert_pressure(pressure),
            humidity,
            battery,
            status_color,
        })
    }
}
