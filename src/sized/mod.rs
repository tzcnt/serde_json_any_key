//! Versions of the map serialization functions for iterators where the size is known beforehand,
//! i.e., those implementing [`ExactSizeIterator`].
//! This is useful when you want to be able to run a binary de/serialization over a struct annotated with `#[serde(with = "any_key_map")]`.

pub mod any_key_map_sized;
pub mod consuming_iter_to_json_sized;
pub mod map_iter_to_json_sized;
