use std::any::TypeId;
use std::cell::Ref;
use std::collections::HashMap;

use log::warn;
use thiserror::Error;

use super::archetype::Archetype;
use super::archetype::ArchetypeError;
use super::archetype::ArchetypeId;
use super::archetype::Component;
use super::archetype::ComponentStore;
use super::bundles::calculate_bundle_id;
use super::bundles::BundleId;
use super::bundles::ComponentBundle;
use super::entities::Entities;
use super::entities::Entity;
use super::entities::EntityArchetypeIndex;
use super::entities::EntityError;
use super::entities::EntityId;
use super::entities::EntityLocation;
use super::helpers::index_twice;
use super::queries::query;
use super::queries::FetchError;
use super::queries::Query;
use super::queries::QueryParameters;

#[derive(Error, Debug)]
pub enum EcsError {
    #[error("Archetype error: {0}")]
    ArchetypeErr(ArchetypeError),
    #[error("Entity error: {0}")]
    EntityErr(EntityError),
    #[error("Query error: {0}")]
    QueryErr(FetchError),
}

pub struct World {
    entities: Entities,
    archetypes: Vec<Archetype>,
    bundle_to_archetype: HashMap<BundleId, ArchetypeId>,
}

impl World {
    fn new() -> Self {
        World {
            entities: Entities::default(),
            archetypes: Vec::new(),
            bundle_to_archetype: HashMap::new(),
        }
    }

    pub(crate) fn add_archetype(&mut self, archetype: Archetype) {
        self.archetypes.push(archetype);
    }

    pub(crate) fn get_archetype(&self, archetype_id: ArchetypeId) -> &Archetype {
        &self.archetypes[archetype_id]
    }

    pub(crate) fn get_archetype_mut(&mut self, archetype_id: ArchetypeId) -> &mut Archetype {
        &mut self.archetypes[archetype_id]
    }

    pub(crate) fn get_bundle_archetype(&self, bundle_id: BundleId) -> Option<&ArchetypeId> {
        self.bundle_to_archetype.get(&bundle_id)
    }

    pub(crate) fn set_bundle_archetype(
        &mut self,
        bundle_id: BundleId,
        archetype_id: ArchetypeId,
    ) -> Option<ArchetypeId> {
        self.bundle_to_archetype.insert(bundle_id, archetype_id)
    }

    pub(crate) fn next_archetype_id(&self) -> usize {
        self.archetypes.len()
    }

    pub(crate) fn add_entity_to_archetype(
        &mut self,
        archetype_id: ArchetypeId,
        entity_id: EntityId,
    ) -> EntityArchetypeIndex {
        self.get_archetype_mut(archetype_id).add_entity(entity_id)
    }

    pub(crate) fn add_component_to_archetype<T: Component>(
        &mut self,
        archetype_id: ArchetypeId,
        component: T,
    ) {
        self.get_archetype_mut(archetype_id)
            .add_entity_component::<T>(component)
    }

    pub(crate) fn set_component_in_archetype<T: Component>(
        &mut self,
        entity_location: &EntityLocation,
        component: T,
    ) {
        self.get_archetype_mut(entity_location.archetype_id)
            .set_entity_component(entity_location.index_in_archetype, component)
            .unwrap();
    }

    /// Spawn an entity with components passed in through a tuple.
    /// Multiple components can be passed in through the tuple.
    /// # Example
    /// ```
    /// # use ecs::*;
    /// let mut world = World::new();
    /// let entity = world.spawn((456, true));
    /// ```
    pub fn spawn(&mut self, bundle: impl ComponentBundle) -> Entity {
        let entity = self.entities.allocate().unwrap();
        let location = bundle.spawn_in_world(self, entity.index);
        self.entities.set_location(entity.index, location).unwrap();
        entity
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities.deallocate(entity).unwrap();
    }

    pub fn entity_count(&self) -> usize {
        self.entities.count()
    }

    pub fn update(&mut self, dt: u32) {
        // Setup stuff
        // Run stuff
        // Cleanup stuff
    }

    /// Add a single component to an entity.
    /// If successful the component is returned.
    /// # Example
    /// ```
    /// # use ecs::*;
    /// let mut world = World::new();
    /// let entity = world.spawn((456, true));
    /// let b = world.add_component(entity, String::from("Name")).unwrap();
    /// ```
    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        if let Some(entry) = self.entities.live_at_index(entity.index).copied() {
            let type_id = TypeId::of::<T>();
            let current_type_ids = self
                .get_archetype(entry.location.archetype_id)
                .components
                .values()
                .map(|comp_store| comp_store.type_id)
                .collect::<Vec<TypeId>>();
            let binary_search_index = current_type_ids.binary_search(&type_id);

            if binary_search_index.is_ok() {
                // Component already exists, just overwrite
                self.set_component_in_archetype(&entry.location, component);
                return Ok(());
            }

            // Component does not exist in the current archetype
            // We'll find one with the right combination or create a new one
            let insert_index = binary_search_index.unwrap_or_else(|i| i);
            let mut new_type_ids = current_type_ids.clone();
            new_type_ids.insert(insert_index, type_id);
            let bundle_id = calculate_bundle_id(&new_type_ids);
            let new_archetype_idx = if let Some(idx) = self.bundle_to_archetype.get(&bundle_id) {
                // Found matching archetype
                *idx
            } else {
                // Didn't find matching archetype, let's create a new one
                let mut new_archetype = Archetype::default();
                for c in self
                    .get_archetype_mut(entry.location.archetype_id)
                    .components
                    .values()
                {
                    new_archetype.components.insert(c.type_id, c.empty_clone());
                }
                let new_archetype_index = self.archetypes.len();
                new_archetype
                    .components
                    .insert(type_id, ComponentStore::new::<T>());
                self.set_bundle_archetype(bundle_id, new_archetype_index);
                self.add_archetype(new_archetype);
                println!("we're creating a new archetype: {}", new_archetype_index);
                new_archetype_index
            };

            println!("old archetype: {}", entry.location.archetype_id);
            // Split borrowing
            let (old_archetype, new_archetype) = index_twice(
                &mut self.archetypes,
                entry.location.archetype_id,
                new_archetype_idx,
            );

            // Basically we're going through this checklist:
            // Add entity to new archetype
            // Update current entity location
            // Migrate components to new archetype
            // Add new component to new archetype too
            // Remove entity from current archetype
            // Update moved entity location, if any

            // Pushes to entity vec, adds space to component sets
            let new_idx_in_archetype = new_archetype.add_entity(entity.index);
            self.entities
                .set_location(
                    entity.index,
                    EntityLocation::new(new_archetype_idx, new_idx_in_archetype),
                )
                .map_err(EcsError::EntityErr)?;

            // Migrate components to new archetype
            for type_id in current_type_ids {
                old_archetype.migrate_component(
                    type_id,
                    entry.location.index_in_archetype,
                    new_archetype,
                );
            }

            // Add new component too
            new_archetype.add_entity_component(component);

            // Update moved entity location, if any
            // We return None if we're last
            if let Some(moved) = old_archetype.remove_entity(entry.location.index_in_archetype) {
                self.entities
                    .set_location(moved, entry.location)
                    .map_err(EcsError::EntityErr)?;
            }
            Ok(())
        } else {
            Err(EcsError::EntityErr(EntityError::DoesNotExist))
        }
    }

    /// Remove a single component from an entity.
    /// If successful the component is returned.
    /// # Example
    /// ```
    /// # use ecs::*;
    /// let mut world = World::new();
    /// let entity = world.spawn((456, true));
    /// let b = world.remove_component::<bool>(entity).unwrap();
    /// ```
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<(), EcsError> {
        if let Some(entry) = self.entities.live_at_index(entity.index).copied() {
            let type_id = TypeId::of::<T>();
            let current_type_ids = self
                .get_archetype(entry.location.archetype_id)
                .components
                .values()
                .map(|comp_store| comp_store.type_id)
                .collect::<Vec<TypeId>>();

            let type_id_idx = current_type_ids.binary_search(&type_id);
            if type_id_idx.is_err() {
                // Component doesn't exist in archetype?!
                return Err(EcsError::ArchetypeErr(ArchetypeError::ComponentMissing));
            }

            let mut new_type_ids = current_type_ids.clone();
            new_type_ids.remove(type_id_idx.unwrap());
            let bundle_id = calculate_bundle_id(&new_type_ids);
            let new_archetype_idx = if let Some(idx) = self.bundle_to_archetype.get(&bundle_id) {
                // Found matching archetype
                *idx
            } else {
                // Didn't find matching archetype, let's create a new one
                let mut new_archetype = Archetype::default();
                for c in self
                    .get_archetype_mut(entry.location.archetype_id)
                    .components
                    .values()
                {
                    new_archetype.components.insert(c.type_id, c.empty_clone());
                }
                let new_archetype_index = self.archetypes.len();
                new_archetype
                    .components
                    .insert(type_id, ComponentStore::new::<T>());
                self.set_bundle_archetype(bundle_id, new_archetype_index);
                self.add_archetype(new_archetype);
                new_archetype_index
            };

            // Basically we're going through this checklist:
            // Add entity to new archetype
            // Update current entity location
            // Migrate components to new archetype, except removed component
            // Remove entity from current archetype
            // Update moved entity location, if any

            let (old_archetype, new_archetype) = index_twice(
                &mut self.archetypes,
                entry.location.archetype_id,
                new_archetype_idx,
            );

            // Pushes into entity vec, adds space to component sets
            let new_idx_in_archetype = new_archetype.add_entity(entity.index);
            self.entities
                .set_location(
                    entity.index,
                    EntityLocation::new(new_archetype_idx, new_idx_in_archetype),
                )
                .map_err(EcsError::EntityErr)?;

            // Migrate components to new archetype, except removed one
            for type_id in new_type_ids {
                old_archetype.migrate_component(
                    type_id,
                    entry.location.index_in_archetype,
                    new_archetype,
                );
            }

            if let Some(moved) = old_archetype.remove_entity(entry.location.index_in_archetype) {
                self.entities
                    .set_location(moved, entry.location)
                    .map_err(EcsError::EntityErr)?;
            }
            Ok(())
        } else {
            Err(EcsError::EntityErr(EntityError::DoesNotExist))
        }
    }

    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if let Some(entry) = self.entities.live_at_index(entity.index).copied() {
            let archetype = self.get_archetype(entry.location.archetype_id);
            return archetype.has_component::<T>();
        }
        false
    }

    pub fn query<'world_borrow, T: QueryParameters>(
        &'world_borrow self,
    ) -> Result<Query<'world_borrow, T>, EcsError> {
        Ok(query::<T>(self)
            .map_err(EcsError::QueryErr)?
            .take()
            .unwrap())
    }

    // pub fn add_system<T: SystemFn>(&mut self, system: T) {}
    // pub fn remove_system<T: SystemFn>(&mut self) -> Option<System> {}
    pub fn has_system() {}
    pub fn get_system() {}
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn can_create_entity() {
        let mut world = World::new();
        let entity = world.spawn(("name", 100));
        assert_eq!(entity.index, 0);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn can_remove_entity() {
        let mut world = World::new();
        struct Health(usize);
        let entity = world.spawn((Health(100),));
        assert_eq!(entity.index, 0);
        assert_eq!(world.entity_count(), 1);

        world.remove(entity);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn can_add_component_to_entity() {
        let mut world = World::new();
        let entity = world.spawn(("name", 120));
        assert_eq!(entity.index, 0);
        assert_eq!(world.entity_count(), 1);

        struct Health(usize);
        struct Name(&'static str);

        world.add_component(entity, Health(100)).unwrap();
        world.add_component(entity, Name("Link")).unwrap();
    }

    #[test]
    fn can_get_entity_component() {
        let mut world = World::new();
        let entity = world.spawn(Name("Link"));
        assert_eq!(entity.index, 0);
        assert_eq!(world.entity_count(), 1);

        struct Health(usize);
        struct Name(&'static str);
        // don't add speed
        struct Speed(usize);

        world.add_component(entity, Health(100));
        world.add_component(entity, Name("Link"));

        let entity_health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(entity_health.0, 100);

        let entity_speed = world.get_component::<Speed>(entity);
        assert!(entity_speed.is_none());
    }
}

//
//     #[test]
//     fn can_iterate_over_components() {
//         let mut world = World::new();
//         let entity = world.spawn();
//         assert_eq!(entity.index(), 0);
//         assert_eq!(world.entity_count(), 1);
//
//         struct Health(usize);
//         struct Name(&'static str);
//         // don't add speed
//         struct Speed(usize);
//
//         world.add_component(entity, Health(100));
//         world.add_component(entity, Name("Link"));
//
//         let entity_health = world.get_component::<Health>(entity).unwrap();
//         assert_eq!(entity_health.0, 100);
//
//         for health in world.query::<Health>().filter_map(|h| h.as_ref()) {
//             assert_eq!(health.0, 100);
//         }
//     }
//
//     #[test]
//     fn can_iterate_mutably_over_components() {
//         let mut world = World::new();
//         let entity = world.spawn();
//         assert_eq!(entity.index(), 0);
//         assert_eq!(world.entity_count(), 1);
//
//         struct Health(usize);
//         struct Name(&'static str);
//         // don't add speed
//         struct Speed(usize);
//
//         world.add_component(entity, Health(100));
//         world.add_component(entity, Name("Link"));
//
//         for health in world.query_mut::<Health>().filter_map(|h| h.as_mut()) {
//             assert_eq!(health.0, 100);
//             health.0 = 120;
//         }
//
//         for health in world.query::<Health>().filter_map(|h| h.as_ref()) {
//             assert_eq!(health.0, 120);
//         }
//     }
//
//     #[test]
//     fn can_iterate_mutably_over_multiple_components() {
//         let mut world = World::new();
//         let entity = world.spawn();
//         assert_eq!(entity.index(), 0);
//         assert_eq!(world.entity_count(), 1);
//
//         struct Health(usize);
//         struct Name(&'static str);
//
//         world.add_component(entity, Health(100));
//         world.add_component(entity, Name("Link"));
//
//         let pools = world.query_mut::<(Health, Name)>();
//
//         for (health, name) in world
//             .query_mut::<(Health, Name)>()
//             .filter_map(|(h, n)| Some((h.as_mut()?, n.as_mut()?)))
//         {
//             assert_eq!(health.0, 100);
//             assert_eq!(name.0, "Link");
//             health.0 = 120;
//             name.0 = "Zelda";
//         }
//         //
//         // for (health, name) in zip.map(|t| (t.0.unwrap(), t.1.unwrap())) {
//         //     assert_eq!(health.0, 120);
//         //     assert_eq!(name.0, "Zelda");
//         // }
//     }
// }
