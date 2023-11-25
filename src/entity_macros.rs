#[macro_export]
macro_rules! add_entity_from_response {
    ($self:expr, $msg:expr) => {
        let d = Box::new($msg.clone()) as Box<dyn prost::Message>;
        let entity = Entity {
            id: $msg.object_id,
            key: $msg.key,
            name: $msg.name,
            icon: $msg.icon,
            category: EntityCategory::try_from($msg.entity_category).unwrap_or_default(),
            unique_id: $msg.unique_id,
            state: ESPHomeMessage::None,
            entity_data: Arc::new(d),
        };

        $self.add_entity(entity).await;
    };
}

#[macro_export]
macro_rules! handle_state_responses {
    ($msg:expr, $m:expr, $entities:expr, $msg_tx:expr) => {{
        let mut entities = $entities.lock().await;
        match entities.update_entity_state($m.key, $msg.clone()) {
            Err(err) => {
                error!("unable to update state of entity {:?}: {:?}", $m.key, err);
            }
            Ok(id) => {
                let app_message = DeviceMessage::DeviceStateChange(Box::new(DaviceState::new(id, $msg.clone())));

                if let Err(err) = $msg_tx.send(app_message).await {
                    error!("unable to send message to Application {:?}", err);
                };
            }
        }
    }};
}
