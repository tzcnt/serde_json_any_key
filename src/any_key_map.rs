/// Apply the attribute `#[serde(with = "any_key_map")]` to de/serialize structs with nested maps that contain non-string keys.
/// 
/// This attribute supports any type that impls `IntoIterator<Item=(&K,&V)>` and `FromIterator<(K,V)>`.
/// ```
/// use std::collections::HashMap;
/// use serde::{Serialize, Deserialize};
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
/// pub struct WithNestedMap {
///   #[serde(with = "any_key_map")]
///   pub struct_map: HashMap<Test, Test>,
///   #[serde(with = "any_key_map")]
///   pub int_map: HashMap<i32, String>
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut data: WithNestedMap = Default::default();
/// data.struct_map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// data.int_map.insert(5, "foo".to_string());
/// 
/// // you can use the usual serde_json functions now
/// let ser = serde_json::to_string(&data).unwrap();
/// let deser: WithNestedMap = serde_json::from_str(&ser).unwrap();
///
/// assert_eq!(data, deser);
/// Ok(()) }
/// try_main().unwrap();
/// ```

use super::*;

/// See docs for [any_key_map](index.html).
pub fn serialize<'s,S,C,K,V>(coll: C, serializer: S) -> Result<S::Ok,S::Error>
where S: Serializer,
C: IntoIterator<Item=(&'s K,&'s V)>,
K: Serialize + Any + 's,
V: Serialize + 's
{
  let mut iter = coll.into_iter();
  let wrap = crate::map_iter_to_json::SerializeMapIterWrapper {
    iter: RefCell::new(&mut iter),
  };
  wrap.serialize(serializer)
}

/// See docs for [any_key_map](index.html).
pub fn deserialize<'d,D,C,K,V>(deserializer: D) -> Result<C, D::Error> where
  D: Deserializer<'d>,
  C: FromIterator<(K,V)> + Sized,
  for<'de> K: Deserialize<'de> + Any + 'd,
  for<'de> V: Deserialize<'de> + 'd,
{
  // any_key_map and any_key_vec use the same deserialize function
  serde_with_utils::deserialize::<'d,D,C,K,V>(deserializer)
}

#[cfg(test)]
mod tests {
  use super::*;
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
      #[serde(with = "any_key_map")]
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
      #[serde(with = "any_key_map")]
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
      #[serde(with = "any_key_map")]
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
