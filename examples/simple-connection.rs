use esphome_client_rs::{
    api::{FanCommandRequest, LightCommandRequest, SubscribeStatesRequest},
    commands::{DeviceCommand, DeviceMessage},
    connection::Connection,
    device::Device,
    messages::*,
};

#[tokio::main]
async fn main() -> esphome_client_rs::result::Result<()> {
    env_logger::builder().init();

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

    // Receive state changes
    subscribe_states(&mut connection).await?;
    'main_loop: loop {
        let msg = connection.wait_for_message().await?;
        eprintln!("Message: {:#?}", msg);
        match msg {
            DeviceMessage::Disconnected => break 'main_loop,
            DeviceMessage::DeviceStateChange(sc) => {
                if sc.name == "light" {
                    if let ESPHomeMessage::LightStateResponse(light_state) = sc.state {
                        set_fan_state(&mut device, &mut connection, light_state.state).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn set_light_state(
    device: &mut Device,
    connection: &mut Connection,
    state: bool,
) -> esphome_client_rs::result::Result<()> {
    eprintln!("set_light_state: {:#?}", state);
    if let Some(key) = device.get_entity_key("light").await {
        connection
            .send_message(DeviceCommand::SendMessage(Box::new(
                ESPHomeMessage::LightCommandRequest(LightCommandRequest {
                    key,
                    has_state: true,
                    state: state,
                    ..Default::default()
                }),
            )))
            .await?;
    }
    Ok(())
}

async fn set_fan_state(
    device: &mut Device,
    connection: &mut Connection,
    state: bool,
) -> esphome_client_rs::result::Result<()> {
    eprintln!("set_fan_state: {:#?}", state);
    if let Some(key) = device.get_entity_key("fan").await {
        connection
            .send_message(DeviceCommand::SendMessage(Box::new(ESPHomeMessage::FanCommandRequest(
                FanCommandRequest {
                    key,
                    has_state: true,
                    state: state,
                    ..Default::default()
                },
            ))))
            .await?;
    }
    Ok(())
}

async fn subscribe_states(connection: &mut Connection) -> esphome_client_rs::result::Result<()> {
    connection
        .send_message(DeviceCommand::SendMessage(Box::new(
            ESPHomeMessage::SubscribeStatesRequest(SubscribeStatesRequest {}),
        )))
        .await?;
    Ok(())
}
