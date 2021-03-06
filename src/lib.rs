/// UUID and protocol constants
pub mod protocol;
pub use protocol::*;

/// Sensor abstraction
pub mod sensor;
pub use sensor::*;

/// Sensor Data abstraction
pub mod data;
pub use data::*;

pub mod error;
pub use error::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn integration() {
        if let Ok(sensor) = SensorManager::init(None).await {
            let cur_readings = sensor.read_current_values().await;
            assert!(cur_readings.is_ok());
            println!("Current Readings: {}", cur_readings.unwrap());
            let update_time = sensor.last_update_time().await;
            println!("Last Update Time: {} seconds", update_time.as_secs());
            // let cur_data = sensor.get_historical_data().await;
            // assert!(cur_data.is_some());
            // println!("Current Data: {}", cur_data.unwrap());
        }
    }
}
