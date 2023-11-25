use crate::messages::ESPHomeMessage;

#[derive(Clone, Debug)]
pub struct DaviceState {
    pub name: String,
    pub state: ESPHomeMessage,
}

impl DaviceState {
    pub fn new(name: String, state: ESPHomeMessage) -> Self {
        Self { name, state }
    }
}

#[derive(Clone, Debug)]
pub enum DeviceMessage {
    Disconnected,
    DeviceStateChange(Box<DaviceState>),
}

#[derive(Clone, Debug)]
pub enum DeviceCommand {
    Stop,
    SendMessage(Box<ESPHomeMessage>),
}
