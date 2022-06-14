## serde_json_any_key
#### [View the docs on docs.rs](https://docs.rs/serde_json_any_key/latest/serde_json_any_key/)  
Workaround for \"key must be a string\" error with serde_json. De/serialize any HashMap<K,V>, Vec<(K,V)>, Iter<(&K,&V)>, or Iter<&(K,V)> as a JSON map.

The output will be the same as if you manually serialized K to a String.
If K already is a String, it will behave identically to serde_json.

Serialization is implemented for any type that implements IntoIterator<Item=(K,V)>, IntoIterator<Item=&(K,V)>, or IntoIterator<Item=(&K,&V)>.  
Deserialization is implemented for any type that implements FromIterator<(K,V)>.

De/serialization of structs with nested maps is supported via the following attributes:  
#[serde(with = "any_key_vec")]  
#[serde(with = "any_key_map")]

All de/serialization is done in a single pass, with no intermediate collection.

This crate is implemented purely using safe, stable Rust.

## Example

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json_any_key::*;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, Debug)]
pub struct Test {
  pub a: i32,
  pub b: i32
}

fn main() {
 // Create a map with a struct key
 let mut map = HashMap::<Test, Test>::new();
 map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
 
 // Regular serde_json cannot serialize this map
 let fail = serde_json::to_string(&map);
 assert_eq!(fail.err().unwrap().to_string(), "key must be a string");
 
 // Use this crate's utility function
 // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
 let ser1 = map.to_json_map().unwrap();
 assert_eq!(ser1, r#"{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}"#);
 
 // You can also serialize a Vec or slice of tuples to a JSON map
 let mut vec = Vec::<(Test, Test)>::new();
 vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
 let ser2 = vec.to_json_map().unwrap();

 // Output is identical in either case
 assert_eq!(ser1, ser2);
 
 // And can be deserialized to either type
 let deser_map: HashMap<Test, Test> = json_to_map(&ser2).unwrap();
 let deser_vec: Vec<(Test, Test)> = json_to_vec(&ser1).unwrap();
 assert_eq!(map, deser_map);
 assert_eq!(vec, deser_vec);
 
 // De/serialization of structs with nested maps is supported via the following attributes:
 // #[serde(with = "any_key_vec")]
 // #[serde(with = "any_key_map")]
 
 // Both the "map" and "vec" fields will serialize identically - as a JSON map
 #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
 pub struct NestedTest {
   #[serde(with = "any_key_map")]
   map: HashMap<Test, Test>,
   #[serde(with = "any_key_vec")]
   vec: Vec<(Test, Test)>
 }
 let nested = NestedTest {
   map: map,
   vec: vec,
 };
 // You can use the usual serde_json functions now
 let ser_nested = serde_json::to_string(&nested).unwrap();
 let deser_nested: NestedTest = serde_json::from_str(&ser_nested).unwrap();
 assert_eq!(nested, deser_nested);
}
```
