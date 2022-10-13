//! ##### Why? : serde_json will not serialize JSON maps where the key is not a string, such as `i32` or `struct` types.  
//! ##### What?: This crate simplifies the process of converting the key to/from a string that serde_json is happy with.  
//! 
//! To serialize a collection, simply call `.to_json_map()`. It's implemented for both [Map-like](trait.MapIterToJson.html#method.to_json_map) and [Vec-like](trait.VecIterToJson.html#method.to_json_map) structures.  
//! There is also a version that consumes/moves out of the collection: [.into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map).
//! 
//! You can deserialize into a [HashMap](fn.json_to_map.html), [Vec of tuples](fn.json_to_vec.html), or [any other collection via Iterator](fn.json_to_iter.html) and the string key will be automatically converted back into the native type.
//! 
//! De/serialization of structs with nested maps is supported via the following attributes:  
//! [#[serde(with = "any_key_map")]](any_key_map/index.html)  
//! [#[serde(with = "any_key_vec")]](any_key_vec/index.html)
//! ```
//! use std::collections::HashMap;
//! use serde::{Serialize, Deserialize};
//! use serde_json::Error;
//! use serde_json_any_key::*;
//! 
//! #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
//! pub struct Test {
//!   pub a: i32,
//!   pub b: i32
//! }
//! 
//! fn try_main() -> Result<(), Error> {
//! 
//! // Create a map with a struct key
//! let mut map = HashMap::<Test, Test>::new();
//! map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
//! 
//! // Regular serde_json cannot serialize this map
//! let fail = serde_json::to_string(&map);
//! assert_eq!(fail.err().unwrap().to_string(), "key must be a string");
//! 
//! // Use this crate's utility function
//! // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
//! let ser1 = map.to_json_map().unwrap();
//! 
//! // You can also serialize a Vec or slice of tuples to a JSON map
//! let mut vec = Vec::<(Test, Test)>::new();
//! vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
//! let ser2 = vec.to_json_map().unwrap();
//!
//! // Output is identical in either case
//! assert_eq!(ser1, ser2);
//! 
//! // And can be deserialized to either type
//! let deser_map: HashMap<Test, Test> = json_to_map(&ser2).unwrap();
//! let deser_vec: Vec<(Test, Test)> = json_to_vec(&ser1).unwrap();
//! assert_eq!(map, deser_map);
//! assert_eq!(vec, deser_vec);
//! 
//! // De/serialization of structs with nested maps is supported via the following attributes:
//! // #[serde(with = "any_key_vec")]
//! // #[serde(with = "any_key_map")]
//! 
//! // Both the "map" and "vec" fields will serialize identically - as a JSON map
//! #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
//! pub struct NestedTest {
//!   #[serde(with = "any_key_map")]
//!   map: HashMap<Test, Test>,
//!   #[serde(with = "any_key_vec")]
//!   vec: Vec<(Test, Test)>
//! }
//! let nested = NestedTest {
//!   map: map,
//!   vec: vec,
//! };
//! // You can use the usual serde_json functions now
//! let ser_nested = serde_json::to_string(&nested).unwrap();
//! let deser_nested: NestedTest = serde_json::from_str(&ser_nested).unwrap();
//! assert_eq!(nested, deser_nested);
//! Ok(()) }
//! try_main().unwrap();
//! ```

// modules
mod json_to_map;
mod json_to_vec;
mod json_to_iter;
mod map_iter_to_json;
mod vec_iter_to_json;
mod consuming_iter_to_json;
mod serde_with_utils;

// exports
pub use json_to_map::json_to_map;
pub use json_to_vec::json_to_vec;
pub use json_to_iter::json_to_iter;
pub use map_iter_to_json::MapIterToJson;
pub use vec_iter_to_json::VecIterToJson;
pub use consuming_iter_to_json::ConsumingIterToJson;
pub mod any_key_map;
pub mod any_key_vec;
