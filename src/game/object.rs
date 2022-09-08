use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use bevy::prelude::{Component, Entity};
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Hash, From, Display, Component,
)]
#[repr(transparent)]
pub struct ObjectId(pub u64);

impl ObjectId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object<T> {
    pub id: ObjectId,
    pub inner: T,
}

impl<T> PartialEq for Object<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Object<T> {}

impl<T> PartialOrd for Object<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}
impl<T> Ord for Object<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> Hash for Object<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Borrow<ObjectId> for Object<T> {
    fn borrow(&self) -> &ObjectId {
        &self.id
    }
}

pub trait WithObjectId {
    fn with_id(self, id: ObjectId) -> Object<Self>
    where
        Self: Sized,
    {
        Object { id, inner: self }
    }
}
impl<T> WithObjectId for T {}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObjectIdGenerator {
    pub last: Option<ObjectId>,
    pub free: Vec<ObjectId>,
}

impl ObjectIdGenerator {
    pub fn next_id(&mut self) -> ObjectId {
        if self.free.is_empty() {
            let next = self.last.unwrap_or_default().next();
            self.last.replace(next);
            next
        } else {
            self.free.pop().unwrap()
        }
    }

    pub fn spawn<T>(&mut self, t: T) -> Object<T> {
        t.with_id(self.next_id())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ObjectEntityMap {
    pub map: HashMap<ObjectId, Entity>,
}
