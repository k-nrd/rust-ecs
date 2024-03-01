use std::{
    any::TypeId,
    marker::PhantomData,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::ecs::{
    archetype::{Archetype, ArchetypeId, Component},
    world::World,
};

use super::error::FetchError;

pub trait QueryParameterFetch<'world_borrow> {
    type FetchItem;

    fn fetch(
        world: &'world_borrow World,
        archetype: ArchetypeId,
    ) -> Result<Self::FetchItem, FetchError>;
}

pub struct QueryParameterFetchRead<T> {
    _data: PhantomData<T>,
}

pub struct QueryParameterFetchWrite<T> {
    _data: PhantomData<T>,
}

impl<'world_borrow, T: Component> QueryParameterFetch<'world_borrow>
    for QueryParameterFetchRead<T>
{
    type FetchItem = RwLockReadGuard<'world_borrow, Vec<T>>;

    fn fetch(
        world: &'world_borrow World,
        archetype_id: ArchetypeId,
    ) -> Result<Self::FetchItem, FetchError> {
        let archetype = world.get_archetype(archetype_id);
        Ok(archetype
            .components
            .get(&TypeId::of::<T>())
            .unwrap()
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>()
            .unwrap()
            .try_read()
            .unwrap())
    }
}

impl<'world_borrow, T: Component> QueryParameterFetch<'world_borrow>
    for QueryParameterFetchWrite<T>
{
    type FetchItem = RwLockWriteGuard<'world_borrow, Vec<T>>;

    fn fetch(
        world: &'world_borrow World,
        archetype_id: ArchetypeId,
    ) -> Result<Self::FetchItem, FetchError> {
        let archetype = world.get_archetype(archetype_id);
        Ok(archetype
            .components
            .get(&TypeId::of::<T>())
            .unwrap()
            .data
            .to_any()
            .downcast_ref::<RwLock<Vec<T>>>()
            .unwrap()
            .try_write()
            .unwrap())
    }
}

/// QueryParameter should fetch its own data, but the data must be requested for any lifetime
/// so an inner trait must be used instead.
/// 'QueryParameter' specifies the nature of the data requested, but not the lifetime.
/// In the future this can (hopefully) be made better with Generic Associated Types.
pub trait QueryParameter {
    type QueryParameterFetch: for<'a> QueryParameterFetch<'a>;
    fn matches_archetype(archetype: &Archetype) -> bool;
}

impl<T: Component> QueryParameter for &T {
    type QueryParameterFetch = QueryParameterFetchRead<T>;
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.has_component::<T>()
    }
}

impl<T: Component> QueryParameter for &mut T {
    type QueryParameterFetch = QueryParameterFetchWrite<T>;
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.has_component::<T>()
    }
}

pub trait QueryParameters: for<'a> QueryParameterFetch<'a> {}
