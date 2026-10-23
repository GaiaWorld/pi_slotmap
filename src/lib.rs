#![doc(html_root_url = "https://docs.rs/slotmap/1.1.1")]
#![crate_name = "pi_slotmap"]
#![cfg_attr(all(nightly, feature = "unstable"), feature(try_reserve))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(all(nightly, doc), feature(doc_cfg))]
#![warn(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_lifetimes,
    unused_import_braces
)]
#![deny(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(renamed_and_removed_lints))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        // Style differences.
        module_name_repetitions,
        redundant_closure_for_method_calls,
        unseparated_literal_suffix,

        // I know what I'm doing and want these.
        wildcard_imports,
        inline_always,
        cast_possible_truncation,
        needless_pass_by_value,

        // Very noisy.
        missing_errors_doc,
        must_use_candidate
    ))]
//! # slotmap
//!
//! This library provides a container with persistent unique keys to access
//! stored values, [`SlotMap`]. Upon insertion a key is returned that can be
//! used to later access or remove the values. Insertion, removal and access all
//! take O(1) time with low overhead. Great for storing collections of objects
//! that need stable, safe references but have no clear ownership otherwise,
//! such as game entities or graph nodes.
//!
//! The difference between a [`BTreeMap`] or [`HashMap`] and a slot map is
//! that the slot map generates and returns the key when inserting a value. A
//! key is always unique and will only refer to the value that was inserted.
//! A slot map's main purpose is to simply own things in a safe and efficient
//! manner.
//!
//! You can also create (multiple) secondary maps that can map the keys returned
//! by [`SlotMap`] to other values, to associate arbitrary data with objects
//! stored in slot maps, without hashing required - it's direct indexing under
//! the hood.
//!
//! The constant-generic `SlotMap` API requires Rust 1.51 or later.
//!
//! # Examples
//!
//! ```
//! # use pi_slotmap::*;
//! let mut sm = SlotMap::new();
//! let foo = sm.insert("foo");  // Key generated on insert.
//! let bar = sm.insert("bar");
//! assert_eq!(sm[foo], "foo");
//! assert_eq!(sm[bar], "bar");
//!
//! sm.remove(bar);
//! let reuse = sm.insert("reuse");  // Space from bar reused.
//! assert_eq!(sm.contains_key(bar), false);  // After deletion a key stays invalid.
//!
//! let mut sec = SecondaryMap::new();
//! sec.insert(foo, "noun");  // We provide the key for secondary maps.
//! sec.insert(reuse, "verb");
//!
//! for (key, val) in sm {
//!     println!("{} is a {}", val, sec[key]);
//! }
//! ```
//!
//! # Serialization through [`serde`], [`no_std`] support and unstable features
//!
//! The maps support (de)serialization through [`serde`] when the `serde`
//! feature is enabled; this also enables serialization for keys from `pi_key`.
//!
//! ```text
//! pi_slotmap = { version = "0.5", features = ["serde"] }
//! ```
//!
//! This crate also supports [`no_std`] environments, but does require the
//! [`alloc`] crate to be available. To enable this you have to disable the
//! `std` feature that is enabled by default:
//!
//! ```text
//! pi_slotmap = { version = "0.5", default-features = false }
//! ```
//!
//! [`SparseSecondaryMap`] and [`DelaySlotMap`] require `std`. The
//! `unstable` feature is retained for compatibility.
//!
//! # Why not index a [`Vec`], or use [`slab`], [`stable-vec`], etc?
//!
//! Those solutions either can not reclaim memory from deleted elements or
//! suffer from the ABA problem. The keys returned by `slotmap` are versioned.
//! This means that once a key is removed, it stays removed, even if the
//! physical storage inside the slotmap is reused for new elements. The key is a
//! permanently unique<sup>*</sup> reference to the inserted value. Despite
//! supporting versioning, a [`SlotMap`] is often not (much) slower than the
//! alternative, by internally using carefully checked unsafe code. Finally,
//! `slotmap` simply has a lot of features that make your life easy.
//!
//! # Performance characteristics and implementation details
//!
//! Insertion, access and deletion is all O(1) with low overhead by storing the
//! elements inside a [`Vec`]. Unlike references or indices into a vector,
//! keys remain valid until removal or an explicit change to their reserved bits. Behind the scenes each
//! slot in the vector is a `(value, version)` tuple. After insertion the
//! returned key also contains a version. Only when the stored version and
//! version in a key match is a key valid. This allows us to reuse space in the
//! vector after deletion without letting removed keys point to spurious new
//! elements. <sup>*</sup>For the default `SlotMap<K, V, 0>`, versions repeat after
//! 2<sup>31</sup> removal/reuse cycles of one slot. With `N` reserved bits and
//! unchanged low bits, the period is 2<sup>31-N</sup> cycles. Trimming trailing
//! slots with `shrink_to_last_occupied` also discards version history and may
//! recreate removed keys on subsequent insertion. A slot map can hold up to
//! 2<sup>32</sup> - 2 elements at a time.
//!
//! The memory usage for each slot in [`SlotMap`] is `4 + max(sizeof(T), 4)`
//! rounded up to the alignment of `T`. Similarly it is `4 + max(sizeof(T), 12)`
//! for [`HopSlotMap`]. [`DenseSlotMap`] has an overhead of 8 bytes per element
//! and 8 bytes per slot.
//!
//! # Choosing [`SlotMap`], [`HopSlotMap`] or [`DenseSlotMap`]
//!
//! A [`SlotMap`] is the fastest for most operations, except iteration. It
//! normally keeps vacant slots to preserve their versions, so iteration may
//! scan many empty slots. `compact()` rebuilds the free list in ascending index
//! order without moving values or dropping versions. `shrink_to_fit()` releases
//! only unused vector capacity. `shrink_to_last_occupied()` additionally removes
//! trailing vacant slots; the caller must handle the lost version history,
//! reserved bits, stale keys, and secondary-map associations.
//!
//! [`HopSlotMap`] solves this by maintaining more information on
//! insertion/removal, allowing it to iterate only over filled slots by 'hopping
//! over' contiguous blocks of vacant slots. This can give it significantly
//! better iteration speed.  If you expect to iterate over all elements in a
//! [`SlotMap`] a lot, and potentially have a lot of deleted elements, choose
//! [`HopSlotMap`]. The downside is that insertion and removal is roughly twice
//! as slow. Random access is the same speed for both.
//!
//! [`DenseSlotMap`] goes even further and stores all elements on a contiguous
//! block of memory. It uses two indirections per random access; the slots
//! contain indices used to access the contiguous memory. This means random
//! access is slower than both [`SlotMap`] and [`HopSlotMap`], but iteration is
//! significantly faster, as fast as a normal [`Vec`].
//!
//! # Reserved version bits in [`SlotMap`]
//!
//! `SlotMap<K, V, const N: u32 = 0>` reserves the low `N` version bits for the
//! caller. Bit `N` is the occupation bit. Removal and reuse each add `1 << N`
//! with wrapping arithmetic, preserving the low bits. `N` must be below 32.
//! Use `SlotMap::<DefaultKey, Value, N>::with_key()` for a nonzero `N`.
//!
//! `set_key_bits(key, bits)` returns the updated key; changing bits makes the
//! old key fail strict lookups. Update any copies and secondary-map entries.
//! `get_key_bits(index)` reads occupied-slot bits without validating generation;
//! reconstructing a key must retain its original high generation bits. Changing
//! bits back can make an earlier key from the same generation match again.
//! Serialization keeps full versions but not `N`; deserialize with the same `N`.
//! These options apply to ordinary `SlotMap`, not `HopSlotMap`, `DenseSlotMap`,
//! or `DelaySlotMap`.
//!
//! # Choosing [`SecondaryMap`] or [`SparseSecondaryMap`]
//!
//! You want to associate extra data with objects stored in a slot map, so you
//! use (multiple) secondary maps to map keys to that data.
//!
//! A [`SecondaryMap`] is simply a [`Vec`] of slots like slot map is, and
//! essentially provides all the same guarantees as [`SlotMap`] does for its
//! operations (with the exception that you provide the keys as produced by the
//! primary slot map). This does mean that even if you associate data to only
//! a single element from the primary slot map, you could need and have to
//! initialize as much memory as the original.
//!
//! A [`SparseSecondaryMap`] is like a [`HashMap`] from keys to objects, however
//! it automatically removes outdated keys for slots that had their space
//! reused. You should use this variant if you expect to store some associated
//! data for only a small portion of the primary slot map.
//!
//! # Custom key types
//!
//! If you have multiple slot maps it's an error to use the key of one slot map
//! on another slot map. The result is safe, but unspecified, and can not be
//! detected at runtime, so it can lead to a hard to find bug.
//!
//! To prevent this, slot maps allow you to specify what the type is of the key
//! they return. You can construct new key types using the [`new_key_type!`]
//! macro. The resulting type behaves exactly like [`DefaultKey`], but is a
//! distinct type. So instead of simply using `SlotMap<DefaultKey, Player>` you
//! would use:
//!
//! ```
//! # use pi_slotmap::*;
//! # #[derive(Copy, Clone)]
//! # struct Player;
//! new_key_type! { struct PlayerKey; }
//! let sm: SlotMap<PlayerKey, Player> = SlotMap::with_key();
//! ```
//!
//! You can write code generic over any key type using the [`Key`] trait.
//!
//! [`Vec`]: std::vec::Vec
//! [`BTreeMap`]: std::collections::BTreeMap
//! [`HashMap`]: std::collections::HashMap
//! [`serde`]: https://github.com/serde-rs/serde
//! [`slab`]: https://crates.io/crates/slab
//! [`stable-vec`]: https://crates.io/crates/stable-vec
//! [`no_std`]: https://doc.rust-lang.org/1.7.0/book/no-stdlib.html

extern crate alloc;

// So our macros can refer to these.
#[doc(hidden)]
pub mod __impl {
    #[cfg(feature = "serde")]
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use core::convert::From;
    pub use core::result::Result;
}

pub mod basic;
pub mod dense;
#[cfg(feature = "std")]
pub mod delay;
pub mod hop;
pub mod secondary;
#[cfg(feature = "std")]
pub mod sparse_secondary;
pub(crate) mod util;


pub use pi_key::{new_key_type, Key, DefaultKey, KeyData, is_older_version};

pub(crate) fn checked_key<K: Key>(index: u32, version: u32) -> K {
    let data = KeyData::new(index, version);
    let key = K::from(data);
    assert_eq!(key.data(), data, "pi_slotmap key must preserve index and generation");
    key
}

#[doc(inline)]
pub use crate::basic::SlotMap;
#[doc(inline)]
pub use crate::dense::DenseSlotMap;
#[cfg(feature = "std")]
#[doc(inline)]
pub use crate::delay::DelaySlotMap;
#[doc(inline)]
pub use crate::hop::HopSlotMap;
#[doc(inline)]
pub use crate::secondary::SecondaryMap;
#[cfg(feature = "std")]
#[doc(inline)]
pub use crate::sparse_secondary::SparseSecondaryMap;

// Keep Slottable for backwards compatibility, but warn about deprecation
// and hide from documentation.
#[doc(hidden)]
#[deprecated(
    since = "1.0.0",
    note = "Slottable is not necessary anymore, slotmap now supports all types on stable."
)]
pub trait Slottable {}

#[doc(hidden)]
#[allow(deprecated)]
impl<T> Slottable for T {}


// pub trait Key = Key1;
// #[doc(hidden)]
// pub type DefaultKey = DefaultKey1;

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    thread_local! {
        static KEY_READS: Cell<usize> = const { Cell::new(0) };
    }

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct ChangingKey {
        data: crate::KeyData,
        alternate: u32,
    }

    impl From<crate::KeyData> for ChangingKey {
        fn from(data: crate::KeyData) -> Self {
            Self { data, alternate: 0 }
        }
    }

    impl pi_null::Null for ChangingKey {
        fn null() -> Self { Self::default() }
        fn is_null(&self) -> bool { self.data.index() == u32::MAX }
    }

    impl crate::Key for ChangingKey {
        fn with(idx: usize) -> Self {
            Self::from(crate::KeyData::new(idx as u32, 0))
        }
        fn data(&self) -> crate::KeyData {
            if self.alternate != 0 && KEY_READS.with(|reads| {
                let n = reads.get();
                reads.set(n + 1);
                n > 0
            }) {
                crate::KeyData::new(self.alternate, self.data.version())
            } else {
                self.data
            }
        }
        fn index(&self) -> usize { self.data.index() as usize }
    }

    #[test]
    fn key_data_preserves_full_slot_version() {
        use super::*;
        let version = (1 << 21) | 1;
        assert_eq!(KeyData::new(1, version).version(), version);
        assert_eq!(checked_key::<DefaultKey>(1, version).data().version(), version);
    }

    #[test]
    fn primary_rejects_index_only_keys_before_value_callback() {
        use super::*;
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let mut basic = SlotMap::<usize, i32>::with_key();
        let invoked = Cell::new(false);
        assert!(catch_unwind(AssertUnwindSafe(|| basic.insert_with_key(|_| {
            invoked.set(true);
            1
        }))).is_err());
        assert!(!invoked.get());
        assert_eq!(basic.len(), 0);
        assert!(basic.is_empty());

        let mut dense = DenseSlotMap::<usize, i32>::with_key();
        invoked.set(false);
        assert!(catch_unwind(AssertUnwindSafe(|| dense.insert_with_key(|_| {
            invoked.set(true);
            1
        }))).is_err());
        assert!(!invoked.get());
        assert_eq!(dense.len(), 0);

        let mut hop = HopSlotMap::<usize, i32>::with_key();
        invoked.set(false);
        assert!(catch_unwind(AssertUnwindSafe(|| hop.insert_with_key(|_| {
            invoked.set(true);
            1
        }))).is_err());
        assert!(!invoked.get());
        assert_eq!(hop.len(), 0);

        let mut delay = DelaySlotMap::<usize, i32>::with_key();
        invoked.set(false);
        assert!(catch_unwind(AssertUnwindSafe(|| delay.insert_with_key(|_| {
            invoked.set(true);
            1
        }))).is_err());
        assert!(!invoked.get());
        assert_eq!(delay.len(), 0);
        assert!(catch_unwind(AssertUnwindSafe(|| delay.reserve_entity())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| unsafe {
            delay.flush(|_, _| invoked.set(true));
        })).is_ok());
        assert!(!invoked.get());
    }

    #[test]
    fn rejected_index_keys_do_not_affect_other_maps() {
        use super::*;
        use std::panic::{catch_unwind, AssertUnwindSafe};
        let mut basic = SlotMap::<DefaultKey, i32>::new();
        let key = basic.insert(1);
        basic.remove(key);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let mut invalid = SlotMap::<usize, i32>::with_key();
            invalid.insert(1);
        })).is_err());
        assert_eq!(basic.len(), 0);

        let mut dense = DenseSlotMap::<DefaultKey, i32>::new();
        let key = dense.insert(1);
        dense.remove(key);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let mut invalid = DenseSlotMap::<usize, i32>::with_key();
            invalid.insert(1);
        })).is_err());
        assert_eq!(dense.len(), 0);

        let mut hop = HopSlotMap::<DefaultKey, i32>::new();
        let key = hop.insert(1);
        hop.remove(key);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let mut invalid = HopSlotMap::<usize, i32>::with_key();
            invalid.insert(1);
        })).is_err());
        assert_eq!(hop.len(), 0);

        let mut delay = DelaySlotMap::<DefaultKey, i32>::new();
        let key = delay.insert(1);
        delay.remove(key);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let mut invalid = DelaySlotMap::<usize, i32>::with_key();
            invalid.insert(1);
        })).is_err());
        assert_eq!(delay.len(), 0);
    }

    #[test]
    fn disjoint_reads_stateful_key_once() {
        use super::*;
        let mut map = SlotMap::<ChangingKey, i32>::with_key();
        let first = map.insert(10);
        let second = map.insert(20);
        let first_data = first.data();
        let second_data = second.data();
        let changing = ChangingKey { data: first_data, alternate: second_data.index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = map.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(map.get(first), Some(&11));
        assert_eq!(map.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(map.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(map.get(first), Some(&11));

        let mut dense = DenseSlotMap::<ChangingKey, i32>::with_key();
        let first = dense.insert(10);
        let second = dense.insert(20);
        let changing = ChangingKey { data: first.data(), alternate: second.data().index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = dense.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(dense.get(first), Some(&11));
        assert_eq!(dense.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(dense.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(dense.get(first), Some(&11));

        let mut hop = HopSlotMap::<ChangingKey, i32>::with_key();
        let first = hop.insert(10);
        let second = hop.insert(20);
        let changing = ChangingKey { data: first.data(), alternate: second.data().index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = hop.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(hop.get(first), Some(&11));
        assert_eq!(hop.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(hop.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(hop.get(first), Some(&11));

        let mut delay = DelaySlotMap::<ChangingKey, i32>::with_key();
        let first = delay.insert(10);
        let second = delay.insert(20);
        let changing = ChangingKey { data: first.data(), alternate: second.data().index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = delay.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(delay.get(first), Some(&11));
        assert_eq!(delay.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(delay.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(delay.get(first), Some(&11));

        let first_data = first.data();
        let second_data = second.data();
        let mut secondary = SecondaryMap::<ChangingKey, i32>::new();
        secondary.insert(first, 10);
        secondary.insert(second, 20);
        let changing = ChangingKey { data: first_data, alternate: second_data.index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = secondary.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(secondary.get(first), Some(&11));
        assert_eq!(secondary.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(secondary.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(secondary.get(first), Some(&11));

        let mut sparse = SparseSecondaryMap::<ChangingKey, i32>::new();
        sparse.insert(first, 10);
        sparse.insert(second, 20);
        let changing = ChangingKey { data: first_data, alternate: second_data.index() };
        KEY_READS.with(|reads| reads.set(0));
        let [a, b] = sparse.get_disjoint_mut([changing, second]).unwrap();
        *a += 1;
        *b += 1;
        assert_eq!(KEY_READS.with(Cell::get), 1);
        assert_eq!(sparse.get(first), Some(&11));
        assert_eq!(sparse.get(second), Some(&21));
        KEY_READS.with(|reads| reads.set(0));
        assert!(sparse.get_disjoint_mut([changing, first]).is_none());
        assert_eq!(sparse.get(first), Some(&11));
    }
    // Intentionally no `use super::*;` because we want to test macro expansion
    // in the *users* scope, which might not have that.
    #[test]
    fn macro_expansion() {
        #![allow(dead_code)]
        use crate::new_key_type;

        // Clobber namespace with clashing names - should still work.
        trait Serialize { }
        trait Deserialize { }
        trait Serializer { }
        trait Deserializer { }
        trait Key { }
        trait From { }
        struct Result;
        struct KeyData;

        new_key_type! {
            struct A;
            pub(crate) struct B;
            pub struct C;
        }
    }

    #[test]
    fn check_is_older_version() {
        use crate::is_older_version;

        let is_older = |a, b| is_older_version(a, b);
        assert!(!is_older(42, 42));
        assert!(is_older(0, 1));
        assert!(!is_older(0, 1 << 31)); // Exactly half a cycle is ambiguous.
        assert!(!is_older(0, (1 << 31) + 1));
        assert!(is_older(u32::MAX, 0));
    }

    #[test]
    fn detached_keys_and_even_secondary_versions() {
        use super::*;
        let mut basic = SlotMap::new();
        let key = basic.insert(3);
        assert_eq!(basic.detach(key), Some(3));
        assert!(!basic.contains_key(key));
        assert_eq!(basic.get(key), None);
        basic.reattach(key, 4);
        assert_eq!(basic.remove(key), Some(4));

        let mut dense = DenseSlotMap::new();
        let key = dense.insert(5);
        assert_eq!(dense.detach(key), Some(5));
        assert_eq!(dense.get(key), None);
        dense.reattach(key, 6);
        assert_eq!(dense.as_slices(), (&[key][..], &[6][..]));

        let even: DefaultKey = KeyData::new(1, 2).into();
        let zero: DefaultKey = KeyData::new(2, 0).into();
        let max: DefaultKey = KeyData::new(3, u32::MAX).into();
        let mut secondary = SecondaryMap::new();
        assert_eq!(secondary.insert(even, 10), None);
        assert_eq!(secondary.insert(zero, 20), None);
        assert_eq!(secondary.insert(max, 30), None);
        assert_eq!(secondary.get_disjoint_mut([even, even]), None);
        assert_eq!(secondary.get_disjoint_mut([max, max]), None);
        let [a, b, c] = secondary.get_disjoint_mut([even, zero, max]).unwrap();
        *a += 1;
        *b += 1;
        *c += 1;
        assert_eq!(secondary.get(even), Some(&11));
        assert_eq!(secondary.get(zero), Some(&21));
        assert_eq!(secondary.get(max), Some(&31));

        let mut sparse = SparseSecondaryMap::new();
        sparse.insert(even, 1);
        sparse.insert(max, 2);
        assert_eq!(sparse.get_disjoint_mut([max, max]), None);
        let [a, b] = sparse.get_disjoint_mut([even, max]).unwrap();
        *a += 3;
        *b += 3;
        assert_eq!(sparse.get(max), Some(&5));
    }

    #[test]
    fn delay_reused_slot_survives_failed_insertion() {
        use super::*;
        let mut map = DelaySlotMap::new();
        let old = map.insert(1);
        assert_eq!(map.remove(old), Some(1));
        assert!(map.try_insert_with_key::<_, ()>(|_| Err(())).is_err());
        let reused = map.insert(2);
        assert_eq!(reused.data().index(), old.data().index());
        assert_eq!(map.get(reused), Some(&2));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn secondary_serde_preserves_even_and_zero_versions() {
        use super::*;
        let even: DefaultKey = KeyData::new(1, 2).into();
        let zero: DefaultKey = KeyData::new(2, 0).into();
        let mut map = SecondaryMap::new();
        map.insert(even, 10);
        map.insert(zero, 20);
        let json = serde_json::to_string(&map).unwrap();
        let restored: SecondaryMap<DefaultKey, i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get(even), Some(&10));
        assert_eq!(restored.get(zero), Some(&20));
        let sparse: SparseSecondaryMap<DefaultKey, i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(sparse.get(zero), Some(&20));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_keeps_keys_embedded_in_values() {
        use super::*;

        let mut map = SlotMap::new();
        let first = map.insert_with_key(|key| (key, 10));
        let second = map.insert((first, 20));
        let json = serde_json::to_string(&map).unwrap();
        let restored: SlotMap<DefaultKey, (DefaultKey, i32)> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get(first), Some(&(first, 10)));
        assert_eq!(restored.get(second), Some(&(first, 20)));
    }

    #[test]
    fn iters_cloneable() {
        use super::*;

        struct NoClone;

        let mut sm = SlotMap::new();
        let mut hsm = HopSlotMap::new();
        let mut dsm = DenseSlotMap::new();
        let mut scm = SecondaryMap::new();
        let mut sscm = SparseSecondaryMap::new();
        scm.insert(sm.insert(NoClone), NoClone);
        sscm.insert(hsm.insert(NoClone), NoClone);
        dsm.insert(NoClone);

        let _ = sm.keys().clone();
        let _ = sm.values().clone();
        let _ = sm.iter().clone();
        let _ = hsm.keys().clone();
        let _ = hsm.values().clone();
        let _ = hsm.iter().clone();
        let _ = dsm.keys().clone();
        let _ = dsm.values().clone();
        let _ = dsm.iter().clone();
        let _ = scm.keys().clone();
        let _ = scm.values().clone();
        let _ = scm.iter().clone();
        let _ = sscm.keys().clone();
        let _ = sscm.values().clone();
        let _ = sscm.iter().clone();
    }

}
