use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json_any_key::*;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Test {
  pub a: i32,
  pub b: i32
}

fn main() {
  let mut map = HashMap::<Test, Test>::new();
  map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});

  // Fails with error: key must be a string
  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error as expected: {}", e); }
  }

  // Long winded workaround that duplicates the map entirely
  // Prints {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  let canonical_serialization = serde_json::to_string(&string_map).unwrap();
  println!("{}", canonical_serialization);
  
  // Use this crate's utility function - elements are serialized lazily
  // Same output
  let serialized = map_to_json(&map).unwrap();
  println!("{}", serialized); // Same output
  assert_eq!(serialized, canonical_serialization);

  // Utility function also exists for vec of tuples
   // Same output
  let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = vec_to_json(&vec).unwrap();
  println!("{}", serialized); // Same output
  assert_eq!(serialized, canonical_serialization);

  // You can also use any other data type that provides an Iter<&(K,V)> or Iter<(&K, &V)>
  // Same output
  let mut btree = std::collections::BTreeMap::<Test, Test>::new();
  btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  let serialized = map_iter_to_json(&mut btree.iter()).unwrap();
  println!("{}", serialized);
  assert_eq!(serialized, canonical_serialization);

  // Also supports deserialization, back to HashMap or Vec
  let deserialized_map: HashMap<Test,Test> = serde_json_any_key::json_to_map(&serialized).unwrap();
  assert_eq!(map, deserialized_map);
  let deserialized_vec: Vec<(Test,Test)> = serde_json_any_key::json_to_vec(&serialized).unwrap();
  assert_eq!(vec, deserialized_vec);

  // If K actually is a String, it will behave identically to serde_json.
  let mut string_map: HashMap<String, i32> = HashMap::new();
  string_map.insert("foo".to_owned(), 1234i32);
  let ser1 = serde_json::to_string(&string_map).unwrap();
  let ser2 = serde_json_any_key::map_to_json(&string_map).unwrap();
  println!("{}", ser2);
  assert_eq!(ser1, ser2);
  let deser1: HashMap<String, i32> = serde_json::from_str(&ser1).unwrap();
  let deser2: HashMap<String, i32> = serde_json_any_key::json_to_map(&ser1).unwrap();
  assert_eq!(deser1, deser2);
}
