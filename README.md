# aranet4-rs

A rust library for the Aranet4 CO2 Sensor

Used by the [`aranet4-dashboard`](https://github.com/lpraneis/aranet4-dashboard) to display current and historical air quality data in a TUI

## Usage

```rust
let sensor = sensor::SensorManager::init(None).await?;
let cur_readings = sensor.read_current_values().await?;
println!("Current Readings: {}", cur_readings);

let update_time = sensor.last_update_time().await?;
println!("Last Update Time: {} seconds", update_time.as_secs());


let cur_data = sensor.get_historical_data().await?;
println!("Current Data: {:?}", cur_data.as_records());
println!("Current Data Metadata: {:?}", cur_data.information);
```

Sample Output:
```
Current Readings: CO2: 475ppm, Temperature: 66.649994F, Pressure : 976kpa, Humidity : 36, Battery: 10, Status Color: 1
Last Update Time: 147 seconds
Current Data: [DataRecord { temperature: 69.08, humidity: 30, pressure: 987.9, co2: 373 }, ... ]
Current Data Metadata: HistoryInformation { interval: Duration { secs: 600, nanos: 0 }, beginning: 2024-04-25T18:08:04.162267335-05:00 }
```
