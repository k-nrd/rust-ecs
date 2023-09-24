use std::{any::TypeId, collections::HashMap};

use bit_set::BitSet;

use super::{entities::Entity, world::Signature};

pub trait SystemFn: 'static {}

pub struct System {
    signature: BitSet,
    entities: Vec<Entity>,
    // This could be an unboxed closure instead,
    // that always takes an iter_mut of entities
    system_fn: Box<dyn SystemFn>,
}

impl System {
    fn new<S: SystemFn>(signature: Signature, system: S) -> Self {
        System {
            signature,
            entities: vec![],
            system_fn: Box::new(system),
        }
    }

    pub fn add_entity_to_system(&mut self, entity: &Entity) {
        self.entities.push(*entity);
    }

    pub fn remove_entity_from_system(&mut self, entity: &Entity) {
        self.entities.retain(|e| e != entity);
    }
    pub fn get_system_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
    pub fn get_system_signature(&self) -> &Signature {
        &self.signature
    }
}

#[derive(Default)]
pub(super) struct Systems {
    active: HashMap<TypeId, System>,
}

impl Systems {
    pub fn add_system<S: SystemFn>(&mut self, system: S) -> Option<S> {
        // self.active
        //     .insert(TypeId::of::<S>(), System::new(signature, system))
        None
    }
    pub fn remove_system<S: SystemFn>(&mut self) -> Option<System> {
        self.active.remove(&TypeId::of::<S>())
    }
    pub fn has_system<S: SystemFn>(&self) -> bool {
        self.active.contains_key(&TypeId::of::<S>())
    }
    pub fn get_system<S: SystemFn>(&mut self) {}
}
