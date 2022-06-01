extern crate serde;
extern crate serde_json;

use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use std::cell::RefCell;
pub struct SerializeMapIterWrapper<'a, K, V>
{
  pub iter: RefCell<&'a mut (dyn Iterator<Item=(&'a K, &'a V)> + 'a)>
}

impl<'a, K, V> Serialize for SerializeMapIterWrapper<'a, K, V> where
  K: Serialize,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    while let Some((k, v)) = iter.next() {
      ser_map.serialize_entry(match &serde_json::to_string(&k)
      {
        Ok(key_string) => key_string,
        Err(e) => { return Err(e).map_err(S::Error::custom); }
      }, &v)?;
    }
    ser_map.end()
  }
}

/// Serialize an Iterator<(&K, &V)> like that given by HashMap::iter().
/// serde_json::to_string() will be called on each K element during serialization.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut map = HashMap::<Test, Test>::new();
/// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// 
/// // Naive serde_json cannot serialize this map.
/// let ser1 = serde_json::to_string(&map);
/// assert_eq!(ser1.err().unwrap().to_string(), "key must be a string");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = serde_json_tuple_iter::map_to_json(&mut map.iter()).unwrap();
///
/// // Compare to a winded workaround that copies the map.
/// // Same output
/// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
/// let ser3 = serde_json::to_string(&string_map).unwrap();
///
/// assert_eq!(ser2, ser3);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn map_to_json<'a, K, V>(iter: &'a mut dyn Iterator<Item=(&'a K, &'a V)>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  serde_json::to_string(&SerializeMapIterWrapper {
    iter: RefCell::new(iter)
  })
}

pub struct SerializeVecIterWrapper<'a, K, V>
{
  pub iter: RefCell<&'a mut (dyn Iterator<Item=&'a (K, V)> + 'a)>
}

impl<'a, K, V> Serialize for SerializeVecIterWrapper<'a, K, V> where
  K: Serialize,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    while let Some((k, v)) = iter.next() {
      ser_map.serialize_entry(match &serde_json::to_string(&k)
      {
        Ok(key_string) => key_string,
        Err(e) => { return Err(e).map_err(S::Error::custom); }
      }, &v)?;
    }
    ser_map.end()
  }
}

/// Serialize an Iterator<&(K, V)> like that given by Vec::<(K, V)>::iter().
/// serde_json::to_string() will be called on each K element during serialization.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let v = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// // Naive serde_json will serialize this as an array, not a map.
/// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
/// let ser1 = serde_json::to_string(&v).unwrap();
/// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = serde_json_tuple_iter::vec_to_json(&mut v.iter()).unwrap();
///
/// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn vec_to_json<'a, K, V>(iter: &'a mut dyn Iterator<Item=&'a (K, V)>) -> Result<String, serde_json::Error> where
K: Serialize,
V: Serialize
{
  serde_json::to_string(&SerializeVecIterWrapper {
    iter: RefCell::new(iter)
  })
}