use bit_set::BitSet;
use std::slice::{Iter, IterMut};

use super::entities::Entity;
use super::{components::Component, entities::Entities};
use super::{components::Components, systems::Systems};

pub type Signature = BitSet;

pub struct World {
    entities: Entities,
    components: Components,
    systems: Systems,
}

impl World {
    fn new() -> Self {
        World {
            entities: Entities::default(),
            components: Components::default(),
            systems: Systems::default(),
        }
    }

    pub fn query<T: Component>(&mut self) -> Iter<'_, Option<T>> {
        self.components.get_component_pool::<T>().unwrap().iter()
    }

    pub fn query_mut<T: Component>(&mut self) -> IterMut<'_, Option<T>> {
        self.components
            .get_component_pool::<T>()
            .unwrap()
            .iter_mut()
    }

    pub fn spawn_entity(&mut self) -> Entity {
        self.components.expand_pools();
        self.entities.spawn_entity()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Entity {
        self.entities.remove_entity(entity)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.count
    }

    pub fn update(dt: u32) {}

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.components.set_entity_component(entity, component);
        self.entities
            .add_to_signature(entity, self.components.get_component_id::<T>());
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        self.entities
            .remove_from_signature(entity, self.components.get_component_id::<T>());
    }

    pub fn has_component<T: Component>(&mut self, entity: Entity) -> bool {
        self.entities
            .signature_contains(entity, self.components.get_component_id::<T>())
    }

    pub fn get_component<T: Component>(&mut self, entity: Entity) -> Option<&T> {
        self.components.get_entity_component(entity)
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_entity_component_mut(entity)
    }

    pub fn add_system() {}
    pub fn remove_system() {}
    pub fn has_system() {}
    pub fn get_system() {}
}

mod tests {
    use super::*;

    #[test]
    fn can_create_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn can_remove_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
        assert_eq!(world.entity_count(), 1);

        world.remove_entity(entity);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn can_add_component_to_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
        assert_eq!(world.entity_count(), 1);

        struct Health(usize);
        struct Name(&'static str);

        world.add_component(entity, Health(100));
        world.add_component(entity, Name("Link"));
    }

    #[test]
    fn can_get_entity_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
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

    #[test]
    fn can_iterate_over_components() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
        assert_eq!(world.entity_count(), 1);

        struct Health(usize);
        struct Name(&'static str);
        // don't add speed
        struct Speed(usize);

        world.add_component(entity, Health(100));
        world.add_component(entity, Name("Link"));

        let entity_health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(entity_health.0, 100);

        for health in world.query::<Health>().filter_map(|h| h.as_ref()) {
            assert_eq!(health.0, 100);
        }
    }

    #[test]
    fn can_iterate_mutably_over_components() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(entity, 0);
        assert_eq!(world.entity_count(), 1);

        struct Health(usize);
        struct Name(&'static str);
        // don't add speed
        struct Speed(usize);

        world.add_component(entity, Health(100));
        world.add_component(entity, Name("Link"));

        let entity_health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(entity_health.0, 100);

        // for health in world.query_mut::<Health>().filter_map(|h| h.as_mut()) {
        //     health.0 = 120;
        // }
        // assert_eq!(entity_health.0, 120);
    }
}
