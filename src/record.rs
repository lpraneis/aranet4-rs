#![allow(dead_code)]
#![allow(unused)]
use crate::SensorError;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono;
use chrono::prelude::*;
use std::fmt;
use std::io::Cursor;

pub struct DataRecord {
    temperature: u16,
    humidity: u8,
    pressure: u8,
    co2: u16,
}
impl fmt::Display for DataRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CO2: {}ppm, Temperature: {}F, Pressure : {}kpa, Humidity : {}",
            self.co2, self.temperature, self.pressure, self.humidity,
        )
    }
}
