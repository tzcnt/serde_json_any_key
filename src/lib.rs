extern crate serde;
extern crate serde_json;

use std::any::{Any, TypeId};
use std::hash::Hash;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use serde::de::{Deserialize};
use std::cell::RefCell;

// https://github.com/rust-lang/rust/issues/49601

pub trait IntoMapIterSerializer<'a,K,V>: IntoIterator<Item=(&'a K,&'a V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeMapIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(&'a K,&'a V)>> IntoMapIterSerializer<'a,K,V> for T where
T: IntoIterator<Item=(&'a K,&'a V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }

struct SerializeMapIterWrapper<'i,'e,K,V>
{
  pub iter: RefCell<&'i mut (dyn Iterator<Item=(&'e K,&'e V)>)>
}

impl<'i,'e,K,V> Serialize for SerializeMapIterWrapper<'i,'e,K,V> where
  K: Serialize + Any,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    if TypeId::of::<K>() == TypeId::of::<String>() {
      while let Some((k, v)) = iter.next() {
        let s = (k as &dyn Any).downcast_ref::<String>().ok_or(S::Error::custom("Failed to serialize String as string"))?;
        ser_map.serialize_entry(s, &v)?;
      }
    } else {
      while let Some((k, v)) = iter.next() {
        ser_map.serialize_entry(match &serde_json::to_string(&k)
        {
          Ok(key_string) => key_string,
          Err(e) => { return Err(e).map_err(S::Error::custom); }
        }, &v)?;
      }
    }
    ser_map.end()
  }
}

/// Serialize an Iterator<(&K, &V)> like that given by HashMap<K,V>::iter().
/// serde_json::to_string() will be called on each K element during serialization.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// use serde_json_any_key::*;
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
/// let ser2 = map_iter_to_json(&mut map.iter()).unwrap();
///
/// // Compare to a long-winded workaround that copies the map.
/// // Same output
/// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
/// let ser3 = serde_json::to_string(&string_map).unwrap();
///
/// assert_eq!(ser2, ser3);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn map_iter_to_json<'i,'e,K,V>(iter: &'i mut dyn Iterator<Item=(&'e K,&'e V)>) -> Result<String, serde_json::Error> where
K: Serialize + Any,
V: Serialize
{
  serde_json::to_string(&SerializeMapIterWrapper {
    iter: RefCell::new(iter)
  })
}

/// A simple wrapper around map_iter_to_json for std::collections::HashMap.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::Serialize;
/// use serde_json::Error;
/// use serde_json_any_key::*;
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
/// let ser1 = map_to_json(&map).unwrap();
/// let ser2 = map_iter_to_json(&mut map.iter()).unwrap();
///
/// assert_eq!(ser1, ser2);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn map_to_json<K,V>(map: &std::collections::HashMap<K,V>) -> Result<String, serde_json::Error> where
K: Serialize + Any,
V: Serialize
{
  map_iter_to_json(&mut map.iter())
}

/// Reverses map_to_json, returning a std::collections::HashMap<K, V>.
///
/// # Examples
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
/// fn try_main() -> Result<(), Error> {
/// let mut map = HashMap::<Test, Test>::new();
/// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// 
/// let ser = map_to_json(&map).unwrap();
/// let deser = json_to_map(&ser).unwrap();
///
/// assert_eq!(map, deser);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn json_to_map<'a,K,V>(str: &'a str) -> Result<std::collections::HashMap<K,V>, serde_json::Error> where
for<'de> K: Deserialize<'de> + std::cmp::Eq + Hash + Any,
for<'de> V: Deserialize<'de>
{
  let mut map: std::collections::HashMap<K,V> = std::collections::HashMap::new();
  let v: serde_json::Value = serde_json::from_str(&str)?;
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a map"))?;
  // handle strings specially as they are not objects
  if TypeId::of::<K>() == TypeId::of::<String>() {
    for (key, val) in o.iter() {
      let key_obj: K = <K as Deserialize>::deserialize(serde_json::Value::from(key.as_str()))?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      map.insert(key_obj, val_obj);
    }
  } else {
    for (key, val) in o.iter() {
      let key_obj: K = serde_json::from_str(key)?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      map.insert(key_obj, val_obj);
    }
  }
  Ok(map)
}

pub trait IntoVecIterSerializer<'a,K,V>: IntoIterator<Item=&'a (K,V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeVecIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=&'a (K,V)>> IntoVecIterSerializer<'a,K,V> for T where
T: IntoIterator<Item=&'a (K,V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }

struct SerializeVecIterWrapper<'i,'e,K,V>
{
  pub iter: RefCell<&'i mut dyn Iterator<Item=&'e (K,V)>>
}

impl<'i,'e,K,V> Serialize for SerializeVecIterWrapper<'i,'e,K,V> where
  K: Serialize + Any,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    if TypeId::of::<K>() == TypeId::of::<String>() {
      while let Some((k, v)) = iter.next() {
        let s = (k as &dyn Any).downcast_ref::<String>().ok_or(S::Error::custom("Failed to serialize String as string"))?;
        ser_map.serialize_entry(s, &v)?;
      }
    } else {
      while let Some((k, v)) = iter.next() {
        ser_map.serialize_entry(match &serde_json::to_string(&k)
        {
          Ok(key_string) => key_string,
          Err(e) => { return Err(e).map_err(S::Error::custom); }
        }, &v)?;
      }
    }
    ser_map.end()
  }
}

/// Serialize an Iterator<&(K, V)> like that given by Vec<(K, V)>::iter().
/// serde_json::to_string() will be called on each K element during serialization.
/// This will produce a JSON Map structure, as if called on a HashMap<K, V>.
///
/// # Examples
/// ``` 
/// use serde::Serialize;
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let vec: Vec<(Test,Test)> = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// // Naive serde_json will serialize this as an array, not a map.
/// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
/// let ser1 = serde_json::to_string(&vec).unwrap();
/// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = vec_iter_to_json(&mut vec.iter()).unwrap();
///
/// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn vec_iter_to_json<'i,'e,K,V>(iter: &'i mut dyn Iterator<Item=&'e (K,V)>) -> Result<String, serde_json::Error> where
K: Serialize + Any,
V: Serialize
{
  serde_json::to_string(&SerializeVecIterWrapper {
    iter: RefCell::new(iter)
  })
}

pub trait IntoConsumingIterSerializer<'a,K,V>: IntoIterator<Item=(K,V)> where
Self: Sized,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  fn into_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeConsumingIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(K,V)>> IntoConsumingIterSerializer<'a,K,V> for T where
T: IntoIterator<Item=(K,V)>,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }

struct SerializeConsumingIterWrapper<'i,K,V>
{
  pub iter: RefCell<&'i mut dyn Iterator<Item=(K,V)>>
}

impl<'i,K,V> Serialize for SerializeConsumingIterWrapper<'i,K,V> where
  K: Serialize + Any,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    if TypeId::of::<K>() == TypeId::of::<String>() {
      while let Some((k, v)) = iter.next() {
        let s = (&k as &dyn Any).downcast_ref::<String>().ok_or(S::Error::custom("Failed to serialize String as string"))?;
        ser_map.serialize_entry(s, &v)?;
      }
    } else {
      while let Some((k, v)) = iter.next() {
        ser_map.serialize_entry(match &serde_json::to_string(&k)
        {
          Ok(key_string) => key_string,
          Err(e) => { return Err(e).map_err(S::Error::custom); }
        }, &v)?;
      }
    }
    ser_map.end()
  }
}

/// Serialize an Iterator<&(K, V)> like that given by Vec<(K, V)>::iter().
/// serde_json::to_string() will be called on each K element during serialization.
/// This will produce a JSON Map structure, as if called on a HashMap<K, V>.
///
/// # Examples
/// ``` 
/// use serde::Serialize;
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let vec: Vec<(Test,Test)> = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// // Naive serde_json will serialize this as an array, not a map.
/// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
/// let ser1 = serde_json::to_string(&vec).unwrap();
/// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
/// 
/// // Use this crate's utility function - elements are serialized lazily.
/// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
/// let ser2 = vec_iter_to_json(&mut vec.iter()).unwrap();
///
/// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn consuming_iter_to_json<'i,K,V>(iter: &'i mut dyn Iterator<Item=(K,V)>) -> Result<String, serde_json::Error> where
K: Serialize + Any,
V: Serialize
{
  serde_json::to_string(&SerializeConsumingIterWrapper {
    iter: RefCell::new(iter)
  })
}

/// A simple wrapper around vec_iter_to_json for std::vec::Vec.
///
/// # Examples
/// ```
/// use serde::Serialize;
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let vec: Vec<(Test,Test)> = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// let ser1 = vec_to_json(&vec).unwrap();
/// let ser2 = vec_iter_to_json(&mut vec.iter()).unwrap();
///
/// assert_eq!(ser1, ser2);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn vec_to_json<K,V>(vec: &Vec<(K,V)>) -> Result<String, serde_json::Error> where
K: Serialize + Any,
V: Serialize
{
  vec_iter_to_json(&mut vec.iter())
}

/// Reverses vec_to_json, returning a Vec<(K, V)>.
///
/// # Examples
/// ```
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
/// fn try_main() -> Result<(), Error> {
/// let vec: Vec<(Test,Test)> = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// let ser = vec_to_json(&vec).unwrap();
/// let deser = json_to_vec(&ser).unwrap();
///
/// assert_eq!(vec, deser);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn json_to_vec<'a,K,V>(str: &'a str) -> Result<Vec<(K,V)>, serde_json::Error> where
for<'de> K: Deserialize<'de> + Any,
for<'de> V: Deserialize<'de>
{
  let mut vec: Vec<(K,V)> = vec![];
  let v: serde_json::Value = serde_json::from_str(&str)?;
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a map"))?;
  // handle strings specially as they are not objects
  if TypeId::of::<K>() == TypeId::of::<String>() {
    for (key, val) in o.iter() {
      let key_obj: K = <K as Deserialize>::deserialize(serde_json::Value::from(key.as_str()))?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      vec.push((key_obj, val_obj));
    }
  } else {
    for (key, val) in o.iter() {
      let key_obj: K = serde_json::from_str(key)?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      vec.push((key_obj, val_obj));
    }
  }
  Ok(vec)
}



#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use serde::{Serialize, Deserialize};

  #[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
  pub struct Test {
    pub a: i32,
    pub b: i32
  }

  #[test]
  fn test_struct_roundtrip_map() {
    let mut data = HashMap::<Test, Test>::new();
    data.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    data.insert(Test {a: 11, b: 12}, Test {a: 13, b: 14});
    let serialized = map_to_json(&data).unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_map_iter() {
    let mut data = HashMap::<Test, Test>::new();
    data.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    data.insert(Test {a: 11, b: 12}, Test {a: 13, b: 14});
    let serialized = map_iter_to_json(&mut data.iter()).unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = vec_to_json(&data).unwrap();
    let mut deser: Vec<(Test, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec_iter() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = vec_iter_to_json(&mut data.iter()).unwrap();
    let mut deser: Vec<(Test, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_canonical_serialization() {
    let mut map = HashMap::<Test, Test>::new();
    map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
    let canonical_serialization = serde_json::to_string(&string_map).unwrap();
    
    let serialized = map_to_json(&map).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
    let serialized = vec_to_json(&vec).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<Test, Test>::new();
    btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let serialized = map_iter_to_json(&mut btree.iter()).unwrap();
    assert_eq!(serialized, canonical_serialization);
  }

  #[test]
  fn test_string_roundtrip_map() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = map_to_json(&data).unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_map_iter() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = map_iter_to_json(&mut data.iter()).unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = vec_to_json(&data).unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec_iter() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = vec_iter_to_json(&mut data.iter()).unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_canonical_serialization() {
    let mut map = HashMap::<String, i32>::new();
    map.insert("foo".to_string(), 5);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map_to_json(&map).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![("foo".to_string(), 5)];
    let serialized = vec_to_json(&vec).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<String, i32>::new();
    btree.insert("foo".to_string(), 5);
    let serialized = map_iter_to_json(&mut btree.iter()).unwrap();
    assert_eq!(serialized, canonical_serialization);
  }


  #[test]
  fn test_int_roundtrip_map() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = map_to_json(&data).unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_map_iter() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = map_iter_to_json(&mut data.iter()).unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = vec_to_json(&data).unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec_iter() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = vec_iter_to_json(&mut data.iter()).unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_canonical_serialization() {
    let mut map = HashMap::<i32, f32>::new();
    map.insert(5, 7.0f32);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map_to_json(&map).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(5, 7.0f32)];
    let serialized = vec_to_json(&vec).unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<i32, f32>::new();
    btree.insert(5, 7.0f32);
    let serialized = map_iter_to_json(&mut btree.iter()).unwrap();
    assert_eq!(serialized, canonical_serialization);
  }

}