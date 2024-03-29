#![doc(html_root_url = "https://docs.rs/slotmap/1.0.6")]
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
    #![feature(trait_alias)]
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
//! The minimum required stable Rust version for this crate is 1.49.
//!
//! # Examples
//!
//! ```
//! # use slotmap::*;
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
//! Both keys and the slot maps have full (de)seralization support through
//! the [`serde`] library. A key remains valid for a slot map even after one or
//! both have been serialized and deserialized! This makes storing or
//! transferring complicated referential structures and graphs a breeze. Care has
//! been taken such that deserializing keys and slot maps from untrusted sources
//! is safe. If you wish to use these features you must enable the `serde`
//! feature flag for `slotmap` in your `Cargo.toml`.
//!
//! ```text
//! slotmap = { version = "1.0", features = ["serde"] }
//! ```
//!
//! This crate also supports [`no_std`] environments, but does require the
//! [`alloc`] crate to be available. To enable this you have to disable the
//! `std` feature that is enabled by default:
//!
//! ```text
//! slotmap = { version = "1.0", default-features = false }
//! ```
//!
//! Unfortunately [`SparseSecondaryMap`] is not available in [`no_std`], because
//! it relies on [`HashMap`]. Finally the `unstable` feature can be defined to
//! enable the parts of `slotmap` that only work on nightly Rust.
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
//! unless you remove a key it is never invalidated. Behind the scenes each
//! slot in the vector is a `(value, version)` tuple. After insertion the
//! returned key also contains a version. Only when the stored version and
//! version in a key match is a key valid. This allows us to reuse space in the
//! vector after deletion without letting removed keys point to spurious new
//! elements. <sup>*</sup>After 2<sup>31</sup> deletions and insertions to the
//! same underlying slot the version wraps around and such a spurious reference
//! could potentially occur. It is incredibly unlikely however, and in all
//! circumstances is the behavior safe. A slot map can hold up to
//! 2<sup>32</sup> - 2 elements at a time.
//!
//! The memory usage for each slot in [`SlotMap`] is `4 + max(sizeof(T), 4)`
//! rounded up to the alignment of `T`. Similarly it is `4 + max(sizeof(T), 12)`
//! for [`HopSlotMap`]. [`DenseSlotMap`] has an overhead of 8 bytes per element
//! and 8 bytes per slot.
//!
//! # Choosing [`SlotMap`], [`HopSlotMap`] or [`DenseSlotMap`]
//!
//! A [`SlotMap`] is the fastest for most operations, except iteration. It can
//! never shrink the size of its underlying storage, because it must remember
//! for each storage slot what the latest stored version was, even if the slot
//! is empty now. This means that iteration can be slow as it must iterate over
//! potentially a lot of empty slots.
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
//! # use slotmap::*;
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
pub mod delay;
pub mod hop;
pub mod secondary;
#[cfg(feature = "std")]
pub mod sparse_secondary;
pub(crate) mod util;


pub use pi_key_alloter::{new_key_type, Key, DefaultKey, KeyData, is_older_version};

#[doc(inline)]
pub use crate::basic::SlotMap;
#[doc(inline)]
pub use crate::dense::DenseSlotMap;
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
        assert!(is_older(0, 1 << 31));
        assert!(!is_older(0, (1 << 31) + 1));
        assert!(is_older(u32::MAX, 0));
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

    #[cfg(feature = "serde")]
    #[test]
    fn key_serde() {
        use super::*;

        // Check round-trip through serde.
        let mut sm = SlotMap::new();
        let k = sm.insert(42);
        let ser = serde_json::to_string(&k).unwrap();
        let de: DefaultKey = serde_json::from_str(&ser).unwrap();
        assert_eq!(k, de);

        // Even if a malicious entity sends up even (unoccupied) versions in the
        // key, we make the version point to the occupied version.
        let malicious: KeyData = serde_json::from_str(&r#"{"idx":0,"version":4}"#).unwrap();
        assert_eq!(malicious.version(), 5);
    }
}
