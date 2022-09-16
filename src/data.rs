use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::Cursor;

use crate::{
    error::SensorError,
    protocol::{convert_pressure, convert_temperature},
};

#[derive(Clone)]
pub struct SensorReadings {
    co2_level: u16,
    temperature: f32,
    pressure: f32,
    humidity: u8,
    battery: u8,
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
    /// construct an empty set of readings. Can be used for displays, etc.
    pub fn empty() -> SensorReadings {
        SensorReadings {
            co2_level: 0,
            temperature: 0.0,
            pressure: 0.0,
            humidity: 0,
            battery: 0,
            status_color: 0,
        }
    }
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
    /// CO2 level, expressed in ppm
    pub fn co2_level(&self) -> u16 {
        self.co2_level
    }
    /// Temperature in Fahrenheit
    pub fn temperature(&self) -> f32 {
        self.temperature
    }
    /// Pressure in kpa
    pub fn pressure(&self) -> f32 {
        self.pressure
    }
    /// Humidity in percent humidity
    pub fn humidity(&self) -> u8 {
        self.humidity
    }
    /// Battery percent
    pub fn battery(&self) -> u8 {
        self.battery
    }
}
