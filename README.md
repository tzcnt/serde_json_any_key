# serde_json_struct_key
Serialize any iterator of 2-tuples into a JSON map with serde_json, even if the first element is a struct type.

This allows you to work around the "key must be a string" error when trying to serialize a HashMap with a struct key. The output will be the same as if you manually serialized the struct key to a string.

```
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json_struct_key::*;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Test {
  pub a: i32,
  pub b: i32
}

fn main() {
  let mut map = HashMap::<Test, Test>::new();
  map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});

  // Fails with Error: key must be a string
  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(serialized) => { println!("{}", serialized); }
    Err(e) => { println!("Error: {}", e); }
  }

  // Long winded workaround that copies the map
  // Prints {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  let serialized = serde_json::to_string(&string_map).unwrap();
  println!("{}", serialized);
  
  // Use this crate's utility function - elements are serialized lazily
  // Same output
  let serialized = serde_json_struct_key::map_iter_to_json(&mut map.iter()).unwrap();
  println!("{}", serialized);

  // Utility function also exists for vec of tuples
  // Same output
  let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = serde_json_struct_key::vec_iter_to_json(&mut vec.iter()).unwrap();
  println!("{}", serialized);

  // You can also use any other data type that provides an Iter<&(K,V)> or Iter<(&K, &V)>
  // Same output
  let mut btree = std::collections::BTreeMap::<Test, Test>::new();
  btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  let serialized = serde_json_struct_key::map_iter_to_json(&mut btree.iter()).unwrap();
  println!("{}", serialized);

  // Also supports deserialization, back to HashMap or Vec
  let deserialized_map: HashMap<Test,Test> = serde_json_struct_key::json_to_map(&serialized).unwrap();
  assert_eq!(map, deserialized_map);
  let deserialized_vec: Vec<(Test,Test)> = serde_json_struct_key::json_to_vec(&serialized).unwrap();
  assert_eq!(vec, deserialized_vec);
}
```

Output:
```
Error: key must be a string
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
```
