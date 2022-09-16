/// UUID and protocol constants
pub mod protocol;

/// Sensor abstraction
pub mod sensor;

/// Sensor Data abstraction
pub mod data;

pub mod error;

pub mod record;

pub mod history;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn integration() {
        let sensor = sensor::SensorManager::init(None)
            .await
            .expect("Cannot create sensor");
        let cur_readings = sensor
            .read_current_values()
            .await
            .expect("Cannot read current values");
        println!("Current Readings: {}", cur_readings);
        let update_time = sensor
            .last_update_time()
            .await
            .expect("Cannot get last update time");
        println!("Last Update Time: {} seconds", update_time.as_secs());
        let cur_data = sensor
            .get_historical_data()
            .await
            .expect("Cannot get historical data");
        println!("Current Data: {:?}", cur_data.as_records());
        println!("Current Data Metadata: {:?}", cur_data.information);
    }
}
