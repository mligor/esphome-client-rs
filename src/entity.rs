use std::{collections::HashMap, sync::Arc};

use crate::{api::EntityCategory, messages::ESPHomeMessage, result::Error, result::Result};

#[derive(Clone, Debug)]
pub struct Entity {
    pub id: String,
    pub key: u32,
    pub name: String,
    pub icon: String,
    pub category: EntityCategory,
    pub unique_id: String,
    pub entity_data: Arc<Box<dyn prost::Message>>,
    pub state: ESPHomeMessage,
}

impl Entity {
    pub fn update_state(&mut self, new_state: ESPHomeMessage) {
        self.state = new_state;
    }
}

pub struct EntityList {
    entities: HashMap<String, Entity>,     // Using String as key type
    entities_by_key: HashMap<u32, String>, // Lookup by key
}

impl EntityList {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            entities_by_key: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.entities_by_key.insert(entity.key, entity.id.clone());
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn get(&self, id: &str) -> Option<Entity> {
        self.entities.get(id).cloned()
    }

    pub fn get_by_key(&self, key: u32) -> Option<Entity> {
        self.entities_by_key
            .get(&key)
            .and_then(|id| self.entities.get(id).cloned())
    }

    pub fn entities(&self) -> HashMap<String, Entity> {
        self.entities.clone()
    }

    pub fn get_key_for_id(&self, id: &str) -> Option<u32> {
        self.get(id).map(|e| e.key)
    }

    // updates the entity state and return ID
    pub fn update_entity_state(&mut self, key: u32, new_state: ESPHomeMessage) -> Result<String> {
        if let Some(id) = self.entities_by_key.get(&key) {
            let e = self.entities.get_mut(id).unwrap();
            e.update_state(new_state);
            Ok(id.clone())
        } else {
            Err(Error::new("unable to find entity"))
        }
    }
}
