#[cfg(not(test))]
use log::info;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
#[cfg(test)]
use std::{println as info, println as warn};

use super::entities::EntityId;

pub type ComponentId = usize;

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

trait ComponentStorage {
    fn push_none(&mut self);
    fn set_none_at(&mut self, index: usize) -> bool;
    fn len(&self) -> usize;
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Component> ComponentStorage for Vec<Option<T>> {
    fn push_none(&mut self) {
        self.push(None);
    }

    fn set_none_at(&mut self, index: usize) -> bool {
        if self.get(index).is_some() {
            self[index] = None;
            return true;
        }
        false
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn to_any(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub(super) struct Components {
    pools: Vec<Box<dyn ComponentStorage>>,
    registry: HashMap<TypeId, ComponentId>,
}

impl Components {
    pub(super) fn expand_pools(&mut self) {
        for pool in self.pools.iter_mut() {
            pool.push_none();
        }
    }

    pub(super) fn clear_entity_components(&mut self, index: EntityId) {
        for pool in self.pools.iter_mut() {
            pool.set_none_at(index);
        }
    }

    fn register_component<T: Component>(&mut self) -> ComponentId {
        // We'll never unregister components, so this is always reliable
        let comp_id = self.pools.len();
        let capacity = self.pools.get(0).map_or(0, |pool| pool.as_ref().len());
        info!("[INFO] Adding new component pool {}", comp_id);
        self.pools
            .push(Box::<Vec<Option<T>>>::new(Vec::with_capacity(capacity)));
        self.registry.insert(TypeId::of::<T>(), comp_id);
        comp_id
    }

    pub(super) fn get_or_register_component_id<T: Component>(&mut self) -> ComponentId {
        match self.registry.get(&TypeId::of::<T>()).copied() {
            Some(id) => id,
            None => self.register_component::<T>(),
        }
    }

    pub(super) fn get_component_id<T: Component>(&self) -> Option<ComponentId> {
        self.registry.get(&TypeId::of::<T>()).copied()
    }

    pub(super) fn get_component_pool<T: Component>(&self) -> Option<&Vec<Option<T>>> {
        for comp_vec in self.pools.iter() {
            if let Some(comp_vec) = comp_vec.to_any().downcast_ref::<Vec<Option<T>>>() {
                return Some(comp_vec);
            }
        }
        None
    }

    pub(super) fn get_component_pool_mut<T: Component>(&mut self) -> Option<&mut Vec<Option<T>>> {
        for comp_vec in self.pools.iter_mut() {
            if let Some(comp_vec) = comp_vec.to_any_mut().downcast_mut::<Vec<Option<T>>>() {
                return Some(comp_vec);
            }
        }
        None
    }

    pub(super) fn get_entity_component<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        if let Some(comp_id) = self.get_component_id::<T>() {
            return self.pools[comp_id]
                .to_any()
                .downcast_ref::<Vec<Option<T>>>()
                .and_then(|comp_vec| comp_vec.get(entity_id))
                .and_then(|opt_t| opt_t.as_ref());
        }
        None
    }

    pub(super) fn get_entity_component_mut<T: Component>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut T> {
        let comp_id = self.get_or_register_component_id::<T>();
        self.pools[comp_id]
            .to_any_mut()
            .downcast_mut::<Vec<Option<T>>>()
            .and_then(|comp_vec| comp_vec.get_mut(entity_id))
            .and_then(|opt_t| opt_t.as_mut())
    }

    pub(super) fn set_entity_component<T: Component>(&mut self, entity_id: EntityId, component: T) {
        let comp_id = self.get_or_register_component_id::<T>();
        info!(
            "[INFO] Adding component {} to entity {}",
            comp_id, entity_id
        );
        let comp_pool = self.pools[comp_id]
            .to_any_mut()
            .downcast_mut::<Vec<Option<T>>>()
            .unwrap();
        info!("[INFO] Component pool length {}", comp_pool.len());
        while (entity_id + 1) >= comp_pool.len() {
            comp_pool.push_none()
        }
        comp_pool[entity_id] = Some(component);
    }
}
