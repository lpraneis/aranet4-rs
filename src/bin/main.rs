#![allow(dead_code)]
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use std::error::Error;
use std::time::Duration;
use tokio::time;

use aranet4::Sensor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .next()
        .expect("Unable to find adapters.");

    central
        .start_scan(ScanFilter::default())
        .await
        .expect("Cannot scan for devices");
    time::sleep(Duration::from_secs(2)).await;

    let sensor = Sensor::init(&central)
        .await
        .expect("Could not create sensor");

    let readings = sensor
        .read_current_values()
        .await
        .expect("Could not read current values");
    println!("{}", readings);
    Ok(())
}
