use std::{
    any::TypeId,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{
    archetype::{Archetype, Component, ComponentStore},
    entities::{EntityId, EntityLocation},
    world::World,
};

pub(crate) type BundleId = u64;

pub trait ComponentBundle: 'static {
    fn new_archetype(&self) -> Archetype;
    fn spawn_in_world(self, world: &mut World, entity_id: EntityId) -> EntityLocation;
}

pub fn calculate_bundle_id(types: &[TypeId]) -> u64 {
    let mut s = DefaultHasher::new();
    types.hash(&mut s);
    s.finish()
}

macro_rules! component_bundle_impl {
    ($($name:tt $index:tt),*) => {
        impl<$($name: Component),*> ComponentBundle for ($($name,)*) {
            fn new_archetype(&self) -> Archetype {
                let mut components = vec![$(ComponentStore::new::<$name>()),*];
                components.sort_unstable_by(|a, b| a.type_id.cmp(&b.type_id));
                Archetype {
                    components: components
                        .into_iter()
                        .map(|comp_store| (comp_store.type_id, comp_store))
                        .collect(),
                    entities: Vec::new(),
                }
            }

            fn spawn_in_world(self, world: &mut World, entity_id: EntityId) -> EntityLocation {
                let mut types = [$(($index, TypeId::of::<$name>())),*];
                types.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                debug_assert!(
                    types.windows(2).all(|x| x[0].1 != x[1].1),
                    "'ComponentBundles' can't have duplicate types"
                );
                let types = [$(types[$index].1),*];
                let bundle_id = calculate_bundle_id(&types);
                let archetype_id = if let Some(id) = world.get_bundle_archetype(bundle_id) {
                    *id
                } else {
                    let archetype = self.new_archetype();
                    let id = world.next_archetype_id();
                    world.set_bundle_archetype(bundle_id, id);
                    world.add_archetype(archetype);
                    id
                };
                let index_in_archetype = world.add_entity_to_archetype(archetype_id, entity_id);
                $(world.add_component_to_archetype(archetype_id, self.$index);)*
                EntityLocation {
                    archetype_id,
                    index_in_archetype,
                }
            }
        }
    };
}

component_bundle_impl!(A 0);
component_bundle_impl!(A 0, B 1);
component_bundle_impl!(A 0, B 1, C 2);
component_bundle_impl!(A 0, B 1, C 2, D 3);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);
component_bundle_impl!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14);
