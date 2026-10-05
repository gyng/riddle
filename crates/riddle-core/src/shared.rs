//! Copy-on-write sharing for state that is cloned far more often than it changes
//! (docs/ITERATION_SPEED.md §0g). The history ring clones the live `Run` and the lineage's
//! facts every `HISTORY_STRIDE` ticks and keeps 31 of them; most of a deep run's clone was a
//! handful of fields that grow a few times a floor (the kills, the episodes, the notes, the
//! facts) and the trace's rows, each re-allocated string by string at every snapshot.
//!
//! `Shared<T>` is a value, not a handle: it reads as `T` (`Deref`), and any `&mut` access
//! (`DerefMut`) first takes a private copy if the value is shared (`Arc::make_mut`), so no
//! write is ever seen through another clone. Equality, `Debug` and serde are `T`'s own (a save
//! or a printout cannot tell a `Shared<T>` from a `T`). A clone is a reference count.
//!
//! The containers keep `#[derive(Clone)]`: a field added later as a plain type is cloned deep,
//! as before — slower, never wrong.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap,BTreeSet};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct Shared<T>(Arc<T>);

impl<T> Shared<T> {
    pub fn new(v: T) -> Self {
        Shared(Arc::new(v))
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(Arc::clone(&self.0))
    }
    fn clone_from(&mut self, source: &Self) {
        // Same value allocation already installed; avoid reference-count churn.
        // This is clone assignment, never an equality shortcut for arbitrary T.
        if !Arc::ptr_eq(&self.0, &source.0) { self.0 = Arc::clone(&source.0); }
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Shared::new(T::default())
    }
}

impl<T> From<T> for Shared<T> {
    fn from(v: T) -> Self {
        Shared::new(v)
    }
}

/// By value, never by pointer: `T`'s own equality, whatever it is.
impl<T: PartialEq> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<T: Eq> Eq for Shared<T> {}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.0, f)
    }
}

impl<T: Serialize> Serialize for Shared<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (*self.0).serialize(s)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Shared<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(Shared::new)
    }
}

impl<'a, T> IntoIterator for &'a Shared<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&*self.0).into_iter()
    }
}

/// `skip_serializing_if` for a shared `Vec` (serde passes the field's own type).
pub fn vec_is_empty<T>(v: &Shared<Vec<T>>) -> bool {
    v.is_empty()
}

/// `skip_serializing_if` for a shared `Option`.
pub fn option_is_none<T>(o: &Shared<Option<T>>) -> bool {
    o.is_none()
}

/// `skip_serializing_if` for a shared `BTreeMap`.
pub fn map_is_empty<K, V>(m: &Shared<BTreeMap<K, V>>) -> bool {
    m.is_empty()
}

impl<A, T: FromIterator<A>> FromIterator<A> for Shared<T> {
    fn from_iter<I: IntoIterator<Item = A>>(it: I) -> Self {
        Shared::new(T::from_iter(it))
    }
}

impl<T> std::borrow::Borrow<T> for Shared<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

/// `skip_serializing_if` for a shared `BTreeSet`.
pub fn set_is_empty<T>(s: &Shared<BTreeSet<T>>) -> bool { s.is_empty() }
