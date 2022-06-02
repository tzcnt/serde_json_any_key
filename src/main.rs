use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json_struct_key::*;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }

  // Long winded workaround that duplicates the map entirely
  // Prints {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  let serialized = serde_json::to_string(&string_map);
  let mut canonical_serialization: Option<String> = None;
  match serialized {
    Ok(s) => { println!("{}", s); canonical_serialization = Some(s); }
    Err(e) => { println!("Error: {}", e); }
  }
  
  // Use this crate's utility function - elements are serialized lazily
  // Same output
  let serialized = map_to_json(&map);
  if let Ok(s) = serialized {
    println!("{}", s); // Same output
    assert!(&s == canonical_serialization.as_ref().unwrap());

    let deser_map: HashMap<Test, Test> = json_to_map(&s).unwrap();
    assert!(deser_map.iter().next().unwrap() == map.iter().next().unwrap());
  } else {
    println!("Error: {}", serialized.expect_err(""));
  }

  // Utility function also exists for vec of tuples
   // Same output
  let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = vec_to_json(&vec);
  if let Ok(s) = serialized {
    println!("{}", s);
    assert!(&s == canonical_serialization.as_ref().unwrap());

    let deser_vec: Vec<(Test, Test)> = json_to_vec(&s).unwrap();
    assert!(deser_vec.iter().next().unwrap() == vec.iter().next().unwrap());
  } else {
    println!("Error: {}", serialized.expect_err(""));
  }

  // You can also use any other data type that provides an Iter<&(K,V)> or Iter<(&K, &V)>
  // Same output
  let mut btree = std::collections::BTreeMap::<Test, Test>::new();
  btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  let serialized = map_iter_to_json(&mut btree.iter());
  if let Ok(s) = serialized {
    println!("{}", s);
    assert!(&s == canonical_serialization.as_ref().unwrap());
  } else {
    println!("Error: {}", serialized.expect_err(""));
  }

}
