extern crate serde;
extern crate serde_json;
mod serde_json_any_map;

use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize, PartialEq, Eq, Hash)]
pub struct Test {
  pub a: i32,
  pub b: i32
}

fn main() {
  let mut map = HashMap::<Test, Test>::new();
  map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});

  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }

  let serialized = serde_json_any_map::to_string(&map);
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }
}
