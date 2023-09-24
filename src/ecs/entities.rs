use bit_set::BitSet;

use super::{components::ComponentId, world::Signature};

pub type Entity = usize;

#[derive(Default)]
pub(super) struct Entities {
    pub(super) count: usize,
    spawn_buffer: Vec<Entity>,
    kill_buffer: Vec<Entity>,
    signatures: Vec<Signature>,
}

impl Entities {
    pub(super) fn add_to_signature(&mut self, entity: Entity, component_id: ComponentId) -> bool {
        if entity >= self.signatures.len() {
            self.signatures.resize_with(entity + 1, BitSet::new);
        }
        self.signatures[entity].insert(component_id)
    }

    pub(super) fn remove_from_signature(
        &mut self,
        entity: Entity,
        component_id: ComponentId,
    ) -> bool {
        self.signatures[entity].remove(component_id)
    }

    pub(super) fn signature_contains(&self, entity: Entity, component_id: ComponentId) -> bool {
        self.signatures[entity].contains(component_id)
    }

    pub(super) fn spawn_entity(&mut self) -> Entity {
        let entity = self.count;
        self.spawn_buffer.push(entity);
        self.count += 1;
        entity
    }

    pub(super) fn remove_entity(&mut self, entity: Entity) -> Entity {
        self.kill_buffer.push(entity);
        self.count -= 1;
        entity
    }

    pub(super) fn has_entity(&self, entity: Entity) {}
}
