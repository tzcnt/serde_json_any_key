// Examples of how to use the library

use std::collections::{BTreeMap, HashMap};
use serde::{Serialize, Deserialize};
use serde_json_any_key::*;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Test {
  pub a: i32,
  pub b: i32
}


#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct NestedTest {
  #[serde(with = "any_key_map")]
  map: HashMap<Test, Test>,
  #[serde(with = "any_key_map")]
  btr: BTreeMap<Test, Test>,
  #[serde(with = "any_key_vec")]
  vec: Vec<(Test, Test)>
}

fn main() {
  let mut map = HashMap::<Test, Test>::new();
  map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});

  // Fails with error: key must be a string
  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(s) => { println!("0 - {}", s); }
    Err(e) => { println!("0 - Error as expected: {}", e); }
  }

  // Long winded workaround that duplicates the map entirely
  // Prints {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  let canonical_serialization = serde_json::to_string(&string_map).unwrap();
  println!("1 - {}", canonical_serialization);
  
  // Use this crate's utility function - elements are serialized lazily
  // Same output
  let serialized = map.to_json_map().unwrap();
  println!("2 - {}", serialized); // Same output
  assert_eq!(serialized, canonical_serialization);

  // Utility function also exists for vec of tuples
   // Same output
  let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = vec.to_json_map().unwrap();
  println!("3 - {}", serialized); // Same output
  assert_eq!(serialized, canonical_serialization);

  // You can also use any other data type that provides an Iter<&(K,V)> or Iter<(&K, &V)>
  // Same output
  let mut btree = BTreeMap::<Test, Test>::new();
  btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  let serialized = btree.to_json_map().unwrap();
  println!("4 - {}", serialized);
  assert_eq!(serialized, canonical_serialization);

  // Also supports deserialization, back to HashMap or Vec
  let deserialized_map: HashMap<Test,Test> = serde_json_any_key::json_to_map(&serialized).unwrap();
  assert_eq!(map, deserialized_map);
  let deserialized_vec: Vec<(Test,Test)> = serde_json_any_key::json_to_vec(&serialized).unwrap();
  assert_eq!(vec, deserialized_vec);

  // Explicitly deserializing a Struct key to a String key will work
  let ds: HashMap<String,Test> = serde_json_any_key::json_to_map(&serialized).unwrap();
  println!("5 - {:?}",ds);
{
  // Collections can be extended using json_to_iter
  let g = serde_json_any_key::json_to_iter::<String,Test>(&serialized).unwrap();
  let mut bt: BTreeMap<String,Test> = BTreeMap::new();
  bt.extend(g.map(|x|x.unwrap()));
  println!("6 - {:?}", bt);
}
{
  // Collections can be extended using json_to_iter
  let g = serde_json_any_key::json_to_iter::<Test,Test>(&serialized).unwrap();
  let mut bt: BTreeMap<Test,Test> = BTreeMap::new();
  bt.extend(g.map(|x|x.unwrap()));
  println!("7 - {:?}", bt);
}

  // If K actually is a String, it will behave identically to serde_json.
  let mut string_map: HashMap<String, i32> = HashMap::new();
  string_map.insert("foo".to_owned(), 1234i32);
  let ser1 = serde_json::to_string(&string_map).unwrap();
  let ser2 = string_map.to_json_map().unwrap();
  println!("8 - {}", ser2);
  assert_eq!(ser1, ser2);
  let deser1: HashMap<String, i32> = serde_json::from_str(&ser1).unwrap();
  let deser2: HashMap<String, i32> = serde_json_any_key::json_to_map(&ser1).unwrap();
  assert_eq!(deser1, deser2);

  // Serialization of structs with nested maps is supported via the following annotations:
  // #[serde(with = "any_key_vec")]
  // #[serde(with = "any_key_map")]
  let mut nested = NestedTest {
    map: Default::default(),
    btr: Default::default(),
    vec: Default::default(),
  };
  nested.map = map;
  nested.vec = vec;
  nested.btr = btree;
  // You can use the usual serde_json functions now
  let serialized = serde_json::to_string(&nested).unwrap();
  println!("9 - {}", serialized);
  let deser: NestedTest = serde_json::from_str(&serialized).unwrap();
  assert_eq!(nested, deser);
}
