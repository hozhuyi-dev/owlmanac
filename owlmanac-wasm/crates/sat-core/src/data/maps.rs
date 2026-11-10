//! Arrays indexed only by `Var` or only by `Lit`, so the two can't be mixed up.

use super::lit::{Lit, Var};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

pub trait Key: Copy {
    fn index(self) -> usize;
}

impl Key for Var {
    fn index(self) -> usize {
        Var::index(self)
    }
}

impl Key for Lit {
    fn index(self) -> usize {
        Lit::index(self)
    }
}

pub type VarMap<T> = KeyMap<Var, T>;
pub type LitMap<T> = KeyMap<Lit, T>;

#[derive(Clone, Debug)]
pub struct KeyMap<K, T> {
    items: Vec<T>,
    key: PhantomData<K>,
}

impl<K: Key, T: Clone> KeyMap<K, T> {
    pub fn new() -> Self {
        KeyMap {
            items: Vec::new(),
            key: PhantomData,
        }
    }

    /// Makes `key` a valid index, filling any new slots with `value`.
    pub fn grow_to_fit(&mut self, key: K, value: T) {
        if key.index() >= self.items.len() {
            self.items.resize(key.index() + 1, value);
        }
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }
}

impl<K: Key, T> Index<K> for KeyMap<K, T> {
    type Output = T;

    fn index(&self, key: K) -> &T {
        &self.items[key.index()]
    }
}

impl<K: Key, T> IndexMut<K> for KeyMap<K, T> {
    fn index_mut(&mut self, key: K) -> &mut T {
        &mut self.items[key.index()]
    }
}
