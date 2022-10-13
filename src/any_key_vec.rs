/// Apply the attribute `#[serde(with = "any_key_vec")]` to de/serialize structs
/// with nested `Vec<(K,V)>` that contain non-string keys.
/// These Vecs will be serialized as JSON maps (as if they were a `HashMap<K,V>`).
/// 
/// This attribute supports any type that impls `IntoIterator<Item=&(K,V)>` and `FromIterator<(K,V)>`.
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
/// pub struct WithNestedVec {
///   #[serde(with = "any_key_vec")]
///   pub structs: Vec<(Test, Test)>,
///   #[serde(with = "any_key_vec")]
///   pub ints: Vec<(i32, String)>
/// }
/// 
/// #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
/// pub struct WithNestedMap {
///   #[serde(with = "any_key_map")]
///   pub structs: HashMap<Test, Test>,
///   #[serde(with = "any_key_map")]
///   pub ints: HashMap<i32, String>
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut vec_data: WithNestedVec = Default::default();
/// vec_data.structs.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
/// vec_data.ints.push((5, "foo".to_string()));
/// 
/// let mut map_data: WithNestedMap = Default::default();
/// map_data.structs.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// map_data.ints.insert(5, "foo".to_string());
/// 
/// // you can use the usual serde_json functions now
/// // both structs produce the same JSON representation
/// let ser_vec = serde_json::to_string(&vec_data).unwrap();
/// let ser_map = serde_json::to_string(&map_data).unwrap();
/// assert_eq!(ser_vec, ser_map);
/// 
/// // and can deserialize into each other
/// let deser_vec: WithNestedVec = serde_json::from_str(&ser_map).unwrap();
/// let deser_map: WithNestedMap = serde_json::from_str(&ser_vec).unwrap();
/// assert_eq!(vec_data, deser_vec);
/// assert_eq!(map_data, deser_map);
/// Ok(()) }
/// try_main().unwrap();
/// ```

use super::*;

/// See docs for [any_key_vec](index.html).
pub fn serialize<'s,S,C,K,V>(coll: C, serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer,
C: IntoIterator<Item=&'s (K,V)>,
K: Serialize + Any + 's,
V: Serialize + 's
{
  let mut iter = coll.into_iter();
  let wrap = SerializeVecIterWrapper {
    iter: RefCell::new(&mut iter),
  };
  wrap.serialize(serializer)
}

/// See docs for [any_key_vec](index.html).
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
  fn test_struct_serde_with_vec() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(Test,Test)>
    }
    let mut data = SerdeWithVec {
      inner: vec![]
    };
    data.inner.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}}");
    let deser: SerdeWithVec = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }
  
  #[test]
  fn test_string_serde_with_vec() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(String, i32)>
    }
    let mut data = SerdeWithVec {
      inner: vec![]
    };
    data.inner.push(("foo".to_string(), 5));
    
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"foo\":5}}");
    let deser: SerdeWithVec = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }
  
  #[test]
  fn test_int_serde_with_vec() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(i32, Test)>
    }
    let mut data = SerdeWithVec {
      inner: vec![]
    };
    data.inner.push((5, Test {a: 6, b: 7}));
    
    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, "{\"inner\":{\"5\":{\"a\":6,\"b\":7}}}");
    let deser: SerdeWithVec = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deser);
  }
}
