//! The attribute `#[serde(with = "any_key_map_json")]` to de/serialize structs with nested maps *of known length*
//! that contain non-string keys, i.e., those implementing ExactSizeIterator.
//! 
//! For further information see [any_key_map](crate::any_key_map).

use crate::serde_with_utils;
use std::any::Any;
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer};

/// See docs for [any_key_map_sized](`crate::any_key_map_sized`).
pub fn serialize<'s,S,C,K,V>(coll: C, serializer: S) -> Result<S::Ok,S::Error>
where S: Serializer,
C: IntoIterator<Item=(&'s K,&'s V)>,
K: Serialize + Any + 's,
V: Serialize + 's,
<C as IntoIterator>::IntoIter: ExactSizeIterator
{
  let mut iter = coll.into_iter();
  let wrap = crate::sized::map_iter_to_json_sized::SerializeMapIterWrapperSized {
    iter: RefCell::new(&mut iter),
  };
  wrap.serialize(serializer)
}

/// See docs for [any_key_map_sized](`crate::any_key_map_sized`).
pub fn deserialize<'d,D,C,K,V>(deserializer: D) -> Result<C, D::Error> where
  D: Deserializer<'d>,
  C: FromIterator<(K,V)> + Sized,
  for<'de> K: Deserialize<'de> + Any + 'd,
  for<'de> V: Deserialize<'de> + 'd,
{
  serde_with_utils::deserialize::<'d,D,C,K,V>(deserializer)
}

#[cfg(test)]
mod tests {
  use crate::any_key_map_sized;
  use std::collections::HashMap;
  use serde::{Serialize, Deserialize};

  #[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
  struct Test {
    pub a: i32,
    pub b: i32
  }

  #[test]
  fn test_struct_serde_with_map() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map_sized")]
      pub inner: HashMap<Test,Test>
    }
    let mut data = SerdeWithMap {
      inner: HashMap::new()
    };
    data.inner.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}}");
    let deser: SerdeWithMap = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }

  
  #[test]
  fn test_string_serde_with_map() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map_sized")]
      pub inner: HashMap<String, i32>
    }
    let mut data = SerdeWithMap {
      inner: HashMap::new()
    };
    data.inner.insert("foo".to_string(), 5);
    
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"foo\":5}}");
    let deser: SerdeWithMap = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }
  
  #[test]
  fn test_int_serde_with_map() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map_sized")]
      pub inner: HashMap<i32, Test>
    }
    let mut data = SerdeWithMap {
      inner: HashMap::new()
    };
    data.inner.insert(5, Test {a: 6, b: 7});
    
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"5\":{\"a\":6,\"b\":7}}}");
    let deser: SerdeWithMap = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }
}
