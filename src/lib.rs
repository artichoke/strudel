/* This is a public domain general purpose hash table package
originally written by Peter Moore @ UCB.

The hash table data structures were redesigned and the package was
rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */

/* The original package implemented classic bucket-based hash tables
   with entries doubly linked for an access by their insertion order.
   To decrease pointer chasing and as a consequence to improve a data
   locality the current implementation is based on storing entries in
   an array and using hash tables with open addressing.  The current
   entries are more compact in comparison with the original ones and
   this also improves the data locality.

   The hash table has two arrays called *bins* and *entries*.

     bins:
    -------
   |       |                  entries array:
   |-------|            --------------------------------
   | index |           |      | entry:  |        |      |
   |-------|           |      |         |        |      |
   | ...   |           | ...  | hash    |  ...   | ...  |
   |-------|           |      | key     |        |      |
   | empty |           |      | record  |        |      |
   |-------|            --------------------------------
   | ...   |                   ^                  ^
   |-------|                   |_ entries start   |_ entries bound
   |deleted|
    -------

   o The entry array contains table entries in the same order as they
     were inserted.

     When the first entry is deleted, a variable containing index of
     the current first entry (*entries start*) is changed.  In all
     other cases of the deletion, we just mark the entry as deleted by
     using a reserved hash value.

     Such organization of the entry storage makes operations of the
     table shift and the entries traversal very fast.

   o The bins provide access to the entries by their keys.  The
     key hash is mapped to a bin containing *index* of the
     corresponding entry in the entry array.

     The bin array size is always power of two, it makes mapping very
     fast by using the corresponding lower bits of the hash.
     Generally it is not a good idea to ignore some part of the hash.
     But alternative approach is worse.  For example, we could use a
     modulo operation for mapping and a prime number for the size of
     the bin array.  Unfortunately, the modulo operation for big
     64-bit numbers are extremely slow (it takes more than 100 cycles
     on modern Intel CPUs).

     Still other bits of the hash value are used when the mapping
     results in a collision.  In this case we use a secondary hash
     value which is a result of a function of the collision bin
     index and the original hash value.  The function choice
     guarantees that we can traverse all bins and finally find the
     corresponding bin as after several iterations the function
     becomes a full cycle linear congruential generator because it
     satisfies requirements of the Hull-Dobell theorem.

     When an entry is removed from the table besides marking the
     hash in the corresponding entry described above, we also mark
     the bin by a special value in order to find entries which had
     a collision with the removed entries.

     There are two reserved values for the bins.  One denotes an
     empty bin, another one denotes a bin for a deleted entry.

   o The length of the bin array is at least two times more than the
     entry array length.  This keeps the table load factor healthy.
     The trigger of rebuilding the table is always a case when we can
     not insert an entry anymore at the entries bound.  We could
     change the entries bound too in case of deletion but than we need
     a special code to count bins with corresponding deleted entries
     and reset the bin values when there are too many bins
     corresponding deleted entries

     Table rebuilding is done by creation of a new entry array and
     bins of an appropriate size.  We also try to reuse the arrays
     in some cases by compacting the array and removing deleted
     entries.

   o To save memory very small tables have no allocated arrays
     bins.  We use a linear search for an access by a key.

   o To save more memory we use 8-, 16-, 32- and 64- bit indexes in
     bins depending on the current hash table size.

   o The implementation takes into account that the table can be
     rebuilt during hashing or comparison functions.  It can happen if
     the functions are implemented in Ruby and a thread switch occurs
     during their execution.

   This implementation speeds up the Ruby hash table benchmarks in
   average by more 40% on Intel Haswell CPU.

*/

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::enum_glob_use)]
#![warn(missing_docs, intra_doc_link_resolution_failure)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![allow(non_camel_case_types)]

//! Insertion-ordered hash table suitable for embedding via FFI.
//!
//! Drop-in replacement for `st_hash` originally written by Peter Moore @ UCB
//! and used in [Ruby](https://github.com/ruby/ruby)'s [implementation][st.c] of
//! the [`Hash`][hash] core class.
//!
//! ## Crate features
//!
//! All features are enabled by default.
//!
//! - **api** - Enables a Rust API that closely mirrors the C API defined in
//!   `ruby/st.h`.
//! - **capi** - Enables a C API suitable for embedding `strudel` with FFI.
//!   Linking in the `libstrudel` cdylib will implement the functions defined in
//!   `ruby/st.h`. Disabling this drops the [`libc`] dependency.
//! - **capi-specialized-init** - Enables additional `st_init_table` C APIs with
//!   known `st_hash_type`s for tables with numeric and string keys.
//!
//! [st.c]: https://github.com/ruby/ruby/blob/v2_6_3/st.c
//! [hash]: https://ruby-doc.org/core-2.6.3/Hash.html

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "capi")]
pub mod capi;
mod entry;
#[cfg(feature = "api")]
mod fnv;
mod hasher;
mod iter;
mod st;
#[cfg(feature = "api")]
mod typedefs;

pub use st::StHashMap;
#[cfg(feature = "api")]
pub use typedefs::*;

pub mod st_hash_map {
    //! An insertion-ordered hash map implemented with [`HashMap`] and
    //! [`BTreeMap`].
    //!
    //! [`HashMap`]: std::collections::HashMap
    //! [`BTreeMap`]: std::collections::BTreeMap

    pub use super::st::*;
}
