use log::{debug, error, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;

use tokio::sync::{mpsc, Mutex};

use crate::commands::DeviceMessage;
use crate::commands::{DaviceState, DeviceCommand};
use crate::messages::{ESPHOmeMessageRead, ESPHomeMessage, ESPHomeMessageWrite};
use crate::{add_entity_from_response, api::*, handle_state_responses};
use crate::{
    connection::Connection,
    entity::{Entity, EntityList},
    result::Result,
};

pub struct Device {
    client_name: String,
    address: String,
    password: Option<String>,
    device_info: Option<DeviceInfoResponse>,
    entities: Arc<Mutex<EntityList>>,
}

impl Device {
    pub fn new(client_name: String, address: String, password: Option<String>) -> Self {
        Self {
            client_name,
            address,
            password,
            device_info: None,
            entities: Arc::new(Mutex::new(EntityList::new())),
        }
    }

    pub fn device_info(&self) -> Result<DeviceInfoResponse> {
        self.device_info
            .clone()
            .ok_or_else(|| "Device information not available".into())
    }

    pub async fn connect(&mut self) -> Result<Connection> {
        let mut stream = tokio::net::TcpStream::connect(&self.address).await?;

        // Handshaking, authorisation and request all infos
        self.send_hello(&mut stream).await?;
        self.send_auth(&mut stream).await?;
        self.send_ping(&mut stream).await?;
        self.enquire_device_info(&mut stream).await?;
        self.list_entries(&mut stream).await?;

        // Create channels
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32); // Application -> Device
        let (msg_tx, msg_rx) = mpsc::channel(32); // Device -> Application

        let (mut esphome_rx, mut esphome_tx) = stream.into_split();

        // listen for commands from application
        tokio::spawn(async move {
            loop {
                let cmd = cmd_rx.recv().await;
                match cmd {
                    Some(cmd) => {
                        debug!("send command to device {:?}", cmd);
                        match cmd {
                            DeviceCommand::Stop => (),
                            DeviceCommand::SendMessage(msg) => {
                                match esphome_tx.write_esphome_message(*msg).await {
                                    Ok(()) => {}
                                    Err(err) => {
                                        error!("error writting message to esphome {:?}", err);
                                    }
                                };
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        // listen for messages from ESPHome
        let entities_clone = self.entities.clone();
        let cmd_tx_clone = cmd_tx.clone();
        tokio::spawn(async move {
            loop {
                let msg = esphome_rx.read_esphome_message().await;
                debug!("message from esphome {:?}", &msg);

                match msg {
                    Ok(msg) => {
                        debug!("message from esphome {:?}", &msg);
                        match &msg {
                            ESPHomeMessage::PingRequest(_) => {
                                if let Err(err) = cmd_tx_clone
                                    .send(DeviceCommand::SendMessage(Box::new(ESPHomeMessage::PingResponse(
                                        PingResponse {},
                                    ))))
                                    .await
                                {
                                    error!("error sending ping response {:?}", err);
                                }
                            }
                            ESPHomeMessage::BinarySensorStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::CoverStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::FanStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::LightStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::SensorStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::SwitchStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::TextSensorStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::CameraImageResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::ClimateStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::NumberStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::SelectStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::LockStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::MediaPlayerStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::AlarmControlPanelStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            ESPHomeMessage::TextStateResponse(m) => {
                                handle_state_responses!(msg, m, entities_clone, msg_tx)
                            }
                            _ => (),
                        }
                    }
                    Err(err) => {
                        error!("error when reading message from esphome {:?}", err);
                        break;
                    }
                };
            }
        });
        Ok(Connection::new(cmd_tx, msg_rx))
    }

    async fn list_entries(&mut self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write_esphome_message(ESPHomeMessage::ListEntitiesRequest(ListEntitiesRequest {}))
            .await?;
        loop {
            let resp = stream.read_esphome_message().await?;
            debug!("got response {:?}", resp);

            if resp.id() == ESPHomeMessage::ListEntitiesDoneResponse(ListEntitiesDoneResponse {}).id() {
                break;
            }

            match resp {
                ESPHomeMessage::ListEntitiesBinarySensorResponse(r) => {
                    add_entity_from_response!(self, r); // Binary Sensor (e.g. Switches, but also other status)
                }
                ESPHomeMessage::ListEntitiesCoverResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesFanResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesLightResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesSensorResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesSwitchResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesTextSensorResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                // ESPHomeMessage::ListEntitiesServicesResponse(_r) => {
                //     //TODO: need custom implementation
                //     //add_entity_from_response!(self, r);
                // }
                ESPHomeMessage::ListEntitiesCameraResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesClimateResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesNumberResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesSelectResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesLockResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesButtonResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesMediaPlayerResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesAlarmControlPanelResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                ESPHomeMessage::ListEntitiesTextResponse(r) => {
                    add_entity_from_response!(self, r);
                }
                _ => {
                    warn!("unexpected response {:?}", resp);
                }
            }
        }
        Ok(())
    }

    async fn send_hello(&self, stream: &mut TcpStream) -> Result<()> {
        let message = ESPHomeMessage::HelloRequest(HelloRequest {
            client_info: self.client_name.to_string(),
            ..Default::default()
        });
        stream.write_esphome_message(message).await?;

        loop {
            let resp = stream.read_esphome_message().await?;
            if let ESPHomeMessage::HelloResponse(r) = resp {
                debug!("hello response received {:?}", r);
                break;
            } else {
                warn!("got unexpected response {:?}", resp);
            }
        }
        Ok(())
    }

    async fn send_auth(&self, stream: &mut TcpStream) -> Result<()> {
        let message = ESPHomeMessage::ConnectRequest(ConnectRequest {
            password: self.password.clone().unwrap_or_default(),
        });
        stream.write_esphome_message(message).await?;
        loop {
            let resp = stream.read_esphome_message().await?;
            if let ESPHomeMessage::ConnectResponse(r) = resp {
                debug!("connect response received {:?}", r);
                break;
            } else {
                warn!("got unexpected response {:?}", resp);
            }
        }
        Ok(())
    }

    async fn send_ping(&self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write_esphome_message(ESPHomeMessage::PingRequest(PingRequest {}))
            .await?;
        loop {
            let resp = stream.read_esphome_message().await?;
            if let ESPHomeMessage::PingResponse(r) = resp {
                debug!("ping response received {:?}", r);
                break;
            } else {
                warn!("got unexpected response {:?}", resp);
            }
        }
        Ok(())
    }

    async fn enquire_device_info(&mut self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write_esphome_message(ESPHomeMessage::DeviceInfoRequest(DeviceInfoRequest {}))
            .await?;
        loop {
            let resp = stream.read_esphome_message().await?;
            if let ESPHomeMessage::DeviceInfoResponse(r) = resp {
                debug!("device info response received {:?}", r);
                self.device_info = Some(r);
                break;
            } else {
                warn!("got unexpected response {:?}", resp);
            }
        }
        Ok(())
    }

    pub async fn add_entity(&mut self, entity: Entity) {
        let mut entities = self.entities.lock().await;
        entities.add(entity);
    }

    pub async fn get_entity(&self, id: &str) -> Option<Entity> {
        let entities = self.entities.lock().await;
        entities.get(id)
    }

    pub async fn entity_by_key(&self, key: u32) -> Option<Entity> {
        let entities = self.entities.lock().await;
        entities.get_by_key(key)
    }

    pub async fn name_by_key(&self, key: u32) -> Option<String> {
        let entities = self.entities.lock().await;
        match entities.get_by_key(key) {
            Some(e) => Some(e.id),
            None => None,
        }
    }

    pub async fn entities(&self) -> HashMap<String, Entity> {
        let entities = self.entities.lock().await;
        entities.entities()
    }

    pub async fn get_entity_key(&self, id: &str) -> Option<u32> {
        let entities = self.entities.lock().await;
        entities.get_key_for_id(id)
    }
}
