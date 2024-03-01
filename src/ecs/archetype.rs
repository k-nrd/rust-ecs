use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use log::debug;
use thiserror::Error;

use super::entities::{EntityArchetypeIndex, EntityId};

#[derive(Debug, Error)]
pub enum ArchetypeError {
    #[error("Archetype does not contain Entity")]
    EntityMissing,
    #[error("Archetype does not contain Component")]
    ComponentMissing,
    #[error("Index exceeds Archetype's entity vec")]
    UnderCapacity,
}

pub type ArchetypeId = usize;

pub type ComponentId = usize;

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

pub trait ComponentSet {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn remove(&mut self, index: usize);
    fn expand(&mut self);
    fn empty_clone(&self) -> Box<dyn ComponentSet>;
    fn migrate(&mut self, index: usize, other_set: &mut dyn ComponentSet);
}

pub type Lock<T> = RwLock<T>;

impl<T: Component> ComponentSet for Lock<Vec<T>> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn len(&self) -> usize {
        self.read().unwrap().len()
    }
    fn remove(&mut self, index: usize) {
        self.get_mut().unwrap().swap_remove(index);
    }
    fn expand(&mut self) {
        self.write().unwrap().reserve(1);
    }
    fn empty_clone(&self) -> Box<dyn ComponentSet> {
        Box::<Lock<Vec<T>>>::default()
    }
    fn migrate(&mut self, index: usize, other_set: &mut dyn ComponentSet) {
        let data: T = { self.get_mut().unwrap().swap_remove(index) };
        component_set_to_mut(other_set).push(data);
    }
}

pub struct ComponentStore {
    pub type_id: TypeId,
    pub data: Box<dyn ComponentSet>,
}

impl ComponentStore {
    pub fn new<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: Box::<Lock<Vec<T>>>::default(),
        }
    }
    pub fn empty_clone(&self) -> ComponentStore {
        ComponentStore {
            type_id: self.type_id,
            data: self.data.empty_clone(),
        }
    }
}

// This could be made unchecked in the future if there's a high degree of confidence in everything else.
fn component_set_to_mut<T: 'static>(c: &mut dyn ComponentSet) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<Lock<Vec<T>>>()
        .unwrap()
        .get_mut()
        .unwrap()
}

// This could be made unchecked in the future if there's a high degree of confidence in everything else.
fn component_set_to_ref<T: 'static>(c: &dyn ComponentSet) -> RwLockReadGuard<'_, Vec<T>> {
    c.to_any()
        .downcast_ref::<Lock<Vec<T>>>()
        .unwrap()
        .try_read()
        .unwrap()
}

#[derive(Default)]
pub struct Archetype {
    pub components: HashMap<TypeId, ComponentStore>,
    pub entities: Vec<EntityId>,
}

impl Archetype {
    /// Gets ComponentSet through its TypeId, downcasts to &mut Vec<T>.
    pub(crate) fn get_component_set_mut<T: Component>(&mut self) -> &mut Vec<T> {
        component_set_to_mut(&mut *self.components.get_mut(&TypeId::of::<T>()).unwrap().data)
    }

    pub(crate) fn has_component<T: Component>(&self) -> bool {
        self.components.get(&TypeId::of::<T>()).is_some()
    }

    /// Should be used to add components for a newly added entity.
    pub(crate) fn add_entity_component<T: Component>(&mut self, component: T) {
        self.get_component_set_mut::<T>().push(component)
    }

    /// Add entity to archetype.
    pub(crate) fn add_entity(&mut self, entity_id: EntityId) -> EntityArchetypeIndex {
        let index = self.entities.len();
        self.entities.push(entity_id);
        for (_, comp_store) in self.components.iter_mut() {
            comp_store.data.expand();
        }
        index
    }

    /// Removes the entity, returns moved entity.
    pub(crate) fn remove_entity(
        &mut self,
        index_in_archetype: EntityArchetypeIndex,
    ) -> Option<EntityId> {
        // We're last, just pop and return None
        if self.entities.len() - 1 == index_in_archetype {
            self.entities.pop();
            return None;
        }
        let moved = self.entities.last().copied();
        self.entities.swap_remove(index_in_archetype);
        moved
    }

    pub(crate) fn set_entity_component<T: Component>(
        &mut self,
        index_in_archetype: EntityArchetypeIndex,
        comp: T,
    ) -> Result<(), ArchetypeError> {
        let comp_store = self.get_component_set_mut::<T>();
        if index_in_archetype >= comp_store.len() {
            return Err(ArchetypeError::UnderCapacity);
        }
        let c = comp_store.get_mut(index_in_archetype).unwrap();
        *c = comp;
        Ok(())
    }

    pub(crate) fn get_entity_component<T: Component>(&self) -> RwLockReadGuard<'_, Vec<T>> {
        component_set_to_ref(&*self.components.get(&TypeId::of::<T>()).unwrap().data)
    }

    pub(crate) fn migrate_component(
        &mut self,
        type_id: TypeId,
        index_in_archetype: EntityArchetypeIndex,
        other_archetype: &mut Archetype,
    ) {
        let other_set = &mut *other_archetype.components.get_mut(&type_id).unwrap().data;
        self.components
            .get_mut(&type_id)
            .expect("")
            .data
            .migrate(index_in_archetype, other_set)
    }
}
