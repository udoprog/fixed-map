// Import the readme as the crate-level docs
// Examples in readme require `map` feature
#![cfg_attr(feature = "map", doc = include_str!("../README.md"))]
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Enable pedantic lints as warnings so we don't break builds when
// lints are modified or new lints are added to clippy.
#![warn(
    // Enable more useful rustc lints
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // Enable pedantic clippy lints
    clippy::pedantic,
    // Useful clippy lints for no_std support
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]
// `clippy::pedantic` exceptions
#![allow(
    // style choice
    clippy::module_name_repetitions,
    // false positive
    clippy::type_repetition_in_bounds,
    // false positive
    clippy::expl_impl_clone_on_copy
)]

pub mod key;

pub mod map;
pub use self::map::Map;

pub mod set;
pub use self::set::Set;

pub mod storage;

#[doc(inline)]
pub use fixed_map_derive::Key;
