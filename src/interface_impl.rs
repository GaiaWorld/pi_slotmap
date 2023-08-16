//!
//! 
use pi_map::Map;

use crate::{SlotMap, Key, SecondaryMap, SparseSecondaryMap, DenseSlotMap, HopSlotMap};

impl<K: Key, V> Map for SlotMap<K, V> {
    type Key = K;

    type Val = V;

    fn len(&self) -> usize {
        self.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        SlotMap::<K, V>::with_capacity_and_key(capacity)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn mem_size(&self) -> usize {
		// TODO
        0
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.get(key.clone())
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.get_mut(key.clone())
    }

    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.get_unchecked(key.clone())
    }

    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.get_unchecked_mut(key.clone())
    }

    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.remove(key.clone()).unwrap()
    }

    fn insert(&mut self, _key: Self::Key, _val: Self::Val) -> Option<Self::Val> {
		// Map应该分为Map和MapMut两个trait， SlotMap没有insert这样的接口， 可以选择不实现MapMut
		todo!()
        // self.insert(key, val)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.remove(key.clone())
    }
}

impl<K: Key, V> Map for DenseSlotMap<K, V> {
    type Key = K;

    type Val = V;

    fn len(&self) -> usize {
        self.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_key(capacity)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn mem_size(&self) -> usize {
		// TODO
        0
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.get(key.clone())
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.get_mut(key.clone())
    }

    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.get_unchecked(key.clone())
    }

    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.get_unchecked_mut(key.clone())
    }

    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.remove(key.clone()).unwrap()
    }

    fn insert(&mut self, _key: Self::Key, _val: Self::Val) -> Option<Self::Val> {
		// Map应该分为Map和MapMut两个trait， SlotMap没有insert这样的接口， 可以选择不实现MapMut
        todo!()
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.remove(key.clone())
    }
}

impl<K: Key, V> Map for HopSlotMap<K, V> {
    type Key = K;

    type Val = V;

    fn len(&self) -> usize {
        self.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_key(capacity)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn mem_size(&self) -> usize {
		// TODO
        0
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.get(key.clone())
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.get_mut(key.clone())
    }

    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.get_unchecked(key.clone())
    }

    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.get_unchecked_mut(key.clone())
    }

    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.remove(key.clone()).unwrap()
    }

    fn insert(&mut self, _key: Self::Key, _val: Self::Val) -> Option<Self::Val> {
		// Map应该分为Map和MapMut两个trait， SlotMap没有insert这样的接口， 可以选择不实现MapMut
        todo!()
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.remove(key.clone())
    }
}

impl<K: Key, V> Map for SecondaryMap<K, V> {
    type Key = K;

    type Val = V;

    fn len(&self) -> usize {
        self.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        SecondaryMap::<K, V>::with_capacity(capacity)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn mem_size(&self) -> usize {
		// TODO
        0
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.get(key.clone())
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.get_mut(key.clone())
    }

    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.get_unchecked(key.clone())
    }

    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.get_unchecked_mut(key.clone())
    }

    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.remove(key.clone()).unwrap()
    }

    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val> {
		// Map应该分为Map和MapMut两个trait， SlotMap没有insert这样的接口， 可以选择不实现MapMut
        self.insert(key, val)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.remove(key.clone())
    }
}

impl<K: Key, V> Map for SparseSecondaryMap<K, V> {
    type Key = K;

    type Val = V;

    fn len(&self) -> usize {
        self.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn mem_size(&self) -> usize {
		// TODO
        0
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key.clone())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.get(key.clone())
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.get_mut(key.clone())
    }

    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.get_unchecked(key.clone())
    }

    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.get_unchecked_mut(key.clone())
    }

    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.remove(key.clone()).unwrap()
    }

    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val> {
		// Map应该分为Map和MapMut两个trait， SlotMap没有insert这样的接口， 可以选择不实现MapMut
        self.insert(key, val)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.remove(key.clone())
    }
}


