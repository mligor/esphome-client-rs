use log::debug;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    commands::{DeviceCommand, DeviceMessage},
    result::Result,
};

pub struct Connection {
    sender: Sender<DeviceCommand>,
    receiver: Receiver<DeviceMessage>,
}

impl Connection {
    pub fn new(sender: Sender<DeviceCommand>, receiver: Receiver<DeviceMessage>) -> Self {
        Self { sender, receiver }
    }

    pub async fn wait_for_message(&mut self) -> Result<DeviceMessage> {
        let msg = self.receiver.recv().await;
        match msg {
            Some(msg) => Ok(msg),
            None => Ok(DeviceMessage::Disconnected),
        }
    }

    pub async fn send_message(&mut self, cmd: DeviceCommand) -> Result<()> {
        debug!("try to message to device {:?}", cmd);
        self.sender.send(cmd).await?;
        Ok(())
    }
}
