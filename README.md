## serde_json_any_key
Workaround for \"key must be a string\" error with serde_json. Serialize any HashMap<K,V>, Vec<(K,V)>, Iter<(&K,&V)>, or Iter<&(K,V)> as a JSON map.
The output will be the same as if you manually serialized K to a String.
If K already is a String, it will behave identically to serde_json.

Also supports deserialization to HashMap<K,V> or Vec<(K,V)>.

## Example

```rust
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

  // The problem this crate aims to solve
  // Fails with error: key must be a string
  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(serialized) => { println!("{}", serialized); }
    Err(e) => { println!("Error as expected: {}", e); }
  }

  // Long winded workaround that copies the map
  // Prints {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  let serialized = serde_json::to_string(&string_map).unwrap();
  println!("{}", serialized);
  
  // Use this crate's utility function - elements are serialized lazily
  // Same output
  let serialized = serde_json_any_key::map_to_json(&map).unwrap();
  println!("{}", serialized);

  // Utility function also exists for vec of tuples
  // Same output
  let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = serde_json_any_key::vec_to_json(&vec).unwrap();
  println!("{}", serialized);

  // Also supports any other data type that generates an Iter<&(K,V)> or Iter<(&K, &V)>
  // Same output
  let mut btree = std::collections::BTreeMap::<Test, Test>::new();
  btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  let serialized = serde_json_any_key::map_iter_to_json(&mut btree.iter()).unwrap();
  println!("{}", serialized);

  // Also supports deserialization, back to HashMap or Vec
  let deserialized_map: HashMap<Test,Test> = serde_json_any_key::json_to_map(&serialized).unwrap();
  assert_eq!(map, deserialized_map);
  let deserialized_vec: Vec<(Test,Test)> = serde_json_any_key::json_to_vec(&serialized).unwrap();
  assert_eq!(vec, deserialized_vec);

  // If K already is a String, it will behave identically to serde_json.
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
```

Output:
```
Error as expected: key must be a string
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"foo":1234}
```
