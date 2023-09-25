use bit_set::BitSet;

use super::{
    components::ComponentId,
    generational_index::{GenerationalIndex, GenerationalIndexAllocator},
    world::Signature,
};

pub type EntityId = usize;

pub type Entity = GenerationalIndex;

#[derive(Default)]
pub(super) struct Entities {
    pub(super) allocator: GenerationalIndexAllocator,
    signatures: Vec<Signature>,
}

impl Entities {
    pub(super) fn count(&self) -> usize {
        self.allocator.live_count()
    }

    pub(super) fn clear_signature(&mut self, entity_id: EntityId) {
        self.signatures[entity_id].clear();
    }

    /// Adds a component to an entity's signature.
    /// Does not check for liveness, so the caller should probably check it.
    pub(super) fn add_to_signature(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> bool {
        self.signatures[entity_id].insert(component_id)
    }

    /// Removes a component from an entity's signature.
    /// Does not check for liveness.
    pub(super) fn remove_from_signature(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> bool {
        self.signatures[entity_id].remove(component_id)
    }

    /// Checks if an entity's signature contains a given component.
    /// Does not check for liveness.
    pub(super) fn signature_contains(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> bool {
        self.signatures[entity_id].contains(component_id)
    }

    /// Allocates a new live entity.
    pub(super) fn spawn_entity(&mut self) -> Entity {
        let entity = self.allocator.allocate();
        if entity.index() >= self.signatures.len() {
            self.signatures.resize_with(entity.index() + 1, BitSet::new);
        }
        entity
    }

    pub(super) fn remove_entity(&mut self, entity: Entity) -> bool {
        self.clear_signature(entity.index());
        self.allocator.deallocate(entity)
    }

    pub(super) fn has_entity(&self, entity: Entity) {}
}
