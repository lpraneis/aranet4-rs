use std::fmt;

#[derive(Debug, Default)]
pub struct DataRecord {
    pub temperature: f32,
    pub humidity: u8,
    pub pressure: f32,
    pub co2: u16,
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
