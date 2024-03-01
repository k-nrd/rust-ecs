use std::marker::PhantomData;

use crate::ecs::world::World;

use super::{
    error::FetchError,
    query_parameters::{QueryParameterFetch, QueryParameters},
};

pub struct QueryFetch<T: QueryParameters> {
    _data: PhantomData<T>,
}

pub struct Query<'world_borrow, T: QueryParameters> {
    data: <T as QueryParameterFetch<'world_borrow>>::FetchItem,
    world: &'world_borrow World,
}

pub fn query<'world_borrow, T: QueryParameters>(
    world: &'world_borrow World,
) -> Result<Option<Query<'world_borrow, T>>, FetchError> {
    Ok(Some(Query {
        data: T::fetch(world, 0)?,
        world,
    }))
}
