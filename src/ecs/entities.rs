use thiserror::Error;

use super::archetype::ArchetypeId;

pub type EntityId = u32;

pub type Generation = u32;

pub type EntityArchetypeIndex = usize;

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("Entity does not exist")]
    DoesNotExist,
    #[error("An entity with this ID was already allocated and is still live")]
    AlreadyAllocated,
    #[error("An entity with this ID was already deallocated and is no longer live")]
    AlreadyDeallocated,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entity {
    pub(crate) index: EntityId,
    pub(crate) generation: Generation,
}

impl Entity {
    pub(crate) fn new(index: EntityId, generation: Generation) -> Self {
        Self { index, generation }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct EntityLocation {
    pub(crate) archetype_id: ArchetypeId,
    pub(crate) index_in_archetype: EntityArchetypeIndex,
}

impl EntityLocation {
    pub fn new(archetype_id: ArchetypeId, index_in_archetype: EntityArchetypeIndex) -> Self {
        Self {
            archetype_id,
            index_in_archetype,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct EntityEntry {
    pub(crate) is_live: bool,
    pub(crate) generation: Generation,
    pub(crate) location: EntityLocation,
}

impl EntityEntry {
    pub(crate) fn new(
        is_live: bool,
        generation: Generation,
        archetype_id: ArchetypeId,
        index_in_archetype: EntityArchetypeIndex,
    ) -> Self {
        Self {
            is_live,
            generation,
            location: EntityLocation::new(archetype_id, index_in_archetype),
        }
    }
}

#[derive(Default)]
pub(crate) struct Entities {
    entries: Vec<EntityEntry>,
    free: Vec<u32>,
}

impl Entities {
    pub(crate) fn len(&self) -> u32 {
        u32::try_from(self.entries.len()).expect("Entries overflow")
    }

    pub(crate) fn allocate(&mut self) -> Result<Entity, EntityError> {
        if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];
            if entry.is_live {
                return Err(EntityError::AlreadyAllocated);
            }
            entry.is_live = true;
            Ok(Entity::new(index, entry.generation))
        } else {
            let generation = 0;
            self.entries.push(EntityEntry::new(true, generation, 0, 0));
            Ok(Entity::new(self.len() - 1, generation))
        }
    }

    pub(crate) fn deallocate(&mut self, entity: Entity) -> Result<(), EntityError> {
        if entity.index >= self.len() {
            return Err(EntityError::DoesNotExist);
        }

        let entry = &mut self.entries[entity.index as usize];
        if !entry.is_live {
            return Err(EntityError::AlreadyDeallocated);
        }

        entry.is_live = false;
        entry.generation = entry
            .generation
            .checked_add(1)
            .expect("EntityEntry overflow");
        self.free.push(entity.index);
        Ok(())
    }

    pub(crate) fn set_location(
        &mut self,
        entity_id: EntityId,
        location: EntityLocation,
    ) -> Result<(), EntityError> {
        if entity_id >= self.len() {
            return Err(EntityError::DoesNotExist);
        }

        let entry = &mut self.entries[entity_id as usize];
        if !entry.is_live {
            return Err(EntityError::AlreadyDeallocated);
        }

        entry.location = location;
        Ok(())
    }

    pub(crate) fn count(&self) -> usize {
        self.entries.iter().filter(|gi| gi.is_live).count()
    }

    pub(crate) fn is_live(&self, entity: Entity) -> bool {
        if entity.index >= self.len() {
            return false;
        }
        self.entries[entity.index as usize].is_live
            && self.entries[entity.index as usize].generation == entity.generation
    }

    pub(crate) fn live_at_index(&self, index: u32) -> Option<&EntityEntry> {
        self.entries.get(index as usize).and_then(|entry| {
            if entry.is_live {
                return Some(entry);
            }
            None
        })
    }

    pub(crate) fn has_entity(&self, entity: Entity) {}
}
