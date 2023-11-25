# esphome-client-rs

## Description
`esphome-client-rs` is a Rust-based client library designed for interacting with ESPHome devices. It offers a simple yet powerful way to connect to, control, and monitor ESPHome devices, leveraging Rust's performance and safety.

## Features
- **Easy Connectivity**: Connect to ESPHome devices effortlessly.
- **Device Control**: Send commands and control ESPHome devices.
- **Event Monitoring**: Listen to and handle events from devices.
- **Rust Efficiency**: Take advantage of Rust's speed and reliability.

## Usage
```rust
  // Create new device
    let mut device = Device::new(
        "my-super-dupper-client".to_string(),
        "10.10.10.10:6053".to_string(),
        Some("mypass".to_string()),
    );

    // Connect to device
    let mut connection = match device.connect().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return Err(e);
        }
    };

    // Get some info out of device
    let info = device.device_info()?;
    eprintln!("Device: {:#?}", info);

    // Get list of entities
    let entities = device.entities().await;
    eprintln!("Entities: {:#?}", entities);

    // Turn the light OFF
    set_light_state(&mut device, &mut connection, false).await?;
    // Turn the fan OFF
    set_fan_state(&mut device, &mut connection, false).await?;
```

## Commercial Use
For commercial use of esphome-client-rs, a separate license is required. Please see the LICENSE file for more details.

## License
This project is licensed under the MIT License - see the LICENSE file for full details.

## Contact
For questions or feedback related to esphome-client-rs, please open an issue on our GitHub repository.

