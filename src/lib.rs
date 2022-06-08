//! ##### TLDR: serde_json will not serialize JSON maps where the key is not a string, e.g. `i32` or `struct` types.  
//! ##### This crate simplifies the process of converting the key to/from a string that serde_json is happy with.  
//! 
//! Serializing is as simple as calling `.to_json_map()` on your data. It's implemented for both [Map-like](trait.MapIterToJson.html#method.to_json_map) and [Vec-like](trait.VecIterToJson.html#method.to_json_map) structures.  
//! There is also a version that consumes/moves out of the data structure: [.into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map).
//! 
//! You can deserialize into a [HashMap](fn.json_to_map.html) or [Vec of tuples](fn.json_to_vec.html), and the string key will be automatically converted back into the native type.
//! ```
//! use std::collections::HashMap;
//! use serde::{Serialize, Deserialize};
//! use serde_json::Error;
//! use serde_json_any_key::*;
//! 
//! #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
//! pub struct Test {
//!   pub a: i32,
//!   pub b: i32
//! }
//! 
//! fn try_main() -> Result<(), Error> {
//! 
//! // Create a map with a struct key
//! let mut map = HashMap::<Test, Test>::new();
//! map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
//! 
//! // Naive serde_json cannot serialize this map.
//! let fail = serde_json::to_string(&map);
//! assert_eq!(fail.err().unwrap().to_string(), "key must be a string");
//! 
//! // Use this crate's utility function
//! // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
//! let ser1 = map.to_json_map().unwrap();
//! 
//! // You can also serialize a Vec or slice of tuples to a JSON map.
//! let mut vec = Vec::<(Test, Test)>::new();
//! vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
//! let ser2 = vec.to_json_map().unwrap();
//!
//! // Output is identical in either case
//! assert_eq!(ser1, ser2);
//! 
//! // And can be deserialized to either type
//! let deser_map: HashMap<Test, Test> = json_to_map(&ser2).unwrap();
//! let deser_vec: Vec<(Test, Test)> = json_to_vec(&ser1).unwrap();
//! assert_eq!(map, deser_map);
//! assert_eq!(vec, deser_vec);
//! Ok(()) }
//! try_main().unwrap();
//! ```

extern crate serde;
extern crate serde_json;

use std::any::{Any, TypeId};
use std::hash::Hash;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use serde::de::{Deserialize};
use serde_json::map::IntoIter;
use std::cell::RefCell;
use std::rc::Rc;

// I'm grateful that I was able to avoid doing it this way:
// https://github.com/rust-lang/rust/issues/49601

pub trait MapIterToJson<'a,K,V>: IntoIterator<Item=(&'a K,&'a V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<(&K, &V)>` to a JSON map. This includes, but is not limited to, the following example types:  
  /// `HashMap<K,V>`  
  /// return type of `HashMap<K,V>::iter()`  
  /// `BTreeMap<K,V>`  
  /// return type of `BTreeMap<K,V>::iter()`
  /// 
  /// To create the JSON map keys, `serde_json::to_string()` will be called on each K element.
  /// 
  /// **This does not consume self**, and is not compatible with consuming iterators, such as those returned by the common
  /// `std::collections::Type::into_iter()` function. For those consuming iterators, call [into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map) instead.
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
  /// let fail = serde_json::to_string(&map);
  /// assert_eq!(fail.err().unwrap().to_string(), "key must be a string");
  /// 
  /// // Use this crate's utility function - elements are serialized lazily.
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser1 = map.to_json_map().unwrap(); // map.iter().to_json_map() is also valid
  ///
  /// // Compare to a long-winded workaround that constructs a new intermediate map.
  /// // Same output
  /// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  /// let ser2 = serde_json::to_string(&string_map).unwrap();
  ///
  /// assert_eq!(ser1, ser2);
  /// Ok(()) }
  /// try_main().unwrap();
  /// ```
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeMapIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(&'a K,&'a V)>> MapIterToJson<'a,K,V> for T where
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

/// Reverses to_json_map(), returning a HashMap<K, V>.
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
/// let ser = map.to_json_map().unwrap();
/// let deser: HashMap<Test, Test> = json_to_map(&ser).unwrap();
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
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a JSON map"))?;
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

pub struct ShrinkWrap<K,V> {
  iter: serde_json::map::IntoIter,
  k: std::marker::PhantomData<K>,
  v: std::marker::PhantomData<V>
}
impl<K,V> ShrinkWrap<K,V> {
  pub fn new(str: &str) -> Self {
    let x = serde_json::from_str(&str).unwrap();
    let z = match x {
      serde_json::Value::Object(map) => Some(map),
            _ => None,
    };
    ShrinkWrap {
      iter: z.unwrap().into_iter(),
      k: std::marker::PhantomData,
      v: std::marker::PhantomData
    }
  }
}

impl<K,V> Iterator for ShrinkWrap<K,V> where
for<'de> K: Deserialize<'de> + std::cmp::Eq + Hash + Any,
for<'de> V: Deserialize<'de>
{
  type Item = Result<(K,V), serde_json::Error>;
  fn next(&mut self) -> Option<Self::Item> {
    let z = self.iter.next();
    if z.is_none() {
      return None;
    }
    let a = z.unwrap();
    let key_obj: K = match serde_json::from_str(&a.0) {
      Ok(k) => k,
      Err(e) => { return Some(Err(e)); }
    };
    let val_obj: V = match <V as Deserialize>::deserialize(a.1) {
      Ok(v) => v,
      Err(e) => { return Some(Err(e)); }
    };
    Some(Ok((key_obj, val_obj)))
  }
}

pub trait VecIterToJson<'a,K,V>: IntoIterator<Item=&'a (K,V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<&(K, V)>` to a JSON map. This includes, but is not limited to, the following example types:  
  /// `Vec<(K,V)>`  
  /// return type of `Vec<(K,V)>::iter()`  
  /// 
  /// To create the JSON map keys, `serde_json::to_string()` will be called on each K element.
  /// 
  /// **This does not consume self**, and is not compatible with consuming iterators, such as those returned by the common
  /// `std::collections::Type::into_iter()` function. For those consuming iterators, call [into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map) instead.
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
  /// let mut vec = Vec::<(Test, Test)>::new();
  /// vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
  /// 
  /// // Naive serde_json will serialize this as an array, not a map.
  /// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
  /// let ser1 = serde_json::to_string(&vec).unwrap();
  /// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
  /// 
  /// // Use this crate's utility function - elements are serialized lazily.
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser2 = vec.to_json_map().unwrap(); // vec.iter().to_json_map() is also valid
  ///
  /// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
  /// Ok(()) }
  /// try_main().unwrap();
  /// ```
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeVecIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=&'a (K,V)>> VecIterToJson<'a,K,V> for T where
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

pub trait ConsumingIterToJson<'a,K,V>: IntoIterator<Item=(K,V)> where
Self: Sized,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<(K, V)>` to a JSON map. This includes, but is not limited to, the following example types:  
  /// `HashMap<K,V>`  
  /// return type of `HashMap<K,V>::into_iter()`  
  /// `Vec<(K,V)>`  
  /// return type of `Vec<(K,V)>::into_iter()`  
  /// 
  /// To create the JSON map keys, `serde_json::to_string()` will be called on each K element.
  /// 
  /// **This consumes self**, and is not compatible with non-consuming iterators, such as those returned by the common
  /// `std::collections::Type::iter()` function. For those non-consuming iterators, call `to_json_map()` instead:  
  /// [For Map-like types](trait.MapIterToJson.html#method.to_json_map)  
  /// [For Vec-like types](trait.VecIterToJson.html#method.to_json_map)
  ///
  /// # Examples
  /// ```
  /// use std::collections::{HashMap, BTreeMap};
  /// use serde::Serialize;
  /// use serde_json::Error;
  /// use serde_json_any_key::*;
  /// 
  /// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
  /// pub struct Test {
  ///   pub a: i32,
  ///   pub b: i32
  /// }
  /// 
  /// fn try_main() -> Result<(), Error> {
  /// let mut map = HashMap::<Test, Test>::new();
  /// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  /// 
  /// let mut btr = BTreeMap::<Test, Test>::new();
  /// btr.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  ///
  /// let mut vec = Vec::<(Test, Test)>::new();
  /// vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
  /// 
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser1 = map.into_json_map().unwrap(); // map.into_iter().into_json_map() is also valid
  /// let ser2 = btr.into_json_map().unwrap(); // btr.into_iter().into_json_map() is also valid
  /// let ser3 = vec.into_json_map().unwrap(); // vec.into_iter().into_json_map() is also valid
  /// 
  /// // map, btr, and vec have all been consumed.
  ///
  /// assert_eq!(ser1, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
  /// assert_eq!(ser1, ser2);
  /// assert_eq!(ser1, ser3);
  /// Ok(()) }
  /// try_main().unwrap();
  /// ```
  fn into_json_map(self) -> Result<String, serde_json::Error> {
    let mut iter = self.into_iter();
    serde_json::to_string(&SerializeConsumingIterWrapper {
      iter: RefCell::new(&mut iter)
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(K,V)>> ConsumingIterToJson<'a,K,V> for T where
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

/// Reverses to_json_map(), returning a Vec<(K, V)>.
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
/// let ser = vec.to_json_map().unwrap();
/// let deser: Vec<(Test,Test)> = json_to_vec(&ser).unwrap();
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
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a JSON map"))?;
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
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_map_iter() {
    let mut data = HashMap::<Test, Test>::new();
    data.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    data.insert(Test {a: 11, b: 12}, Test {a: 13, b: 14});
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(Test, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec_iter() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = data.iter().to_json_map().unwrap();
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
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<Test, Test>::new();
    btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }

  #[test]
  fn test_string_roundtrip_map() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_map_iter() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec_iter() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = data.iter().to_json_map().unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_canonical_serialization() {
    let mut map = HashMap::<String, i32>::new();
    map.insert("foo".to_string(), 5);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![("foo".to_string(), 5)];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<String, i32>::new();
    btree.insert("foo".to_string(), 5);
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }


  #[test]
  fn test_int_roundtrip_map() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_map_iter() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec_iter() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = data.iter().to_json_map().unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_canonical_serialization() {
    let mut map = HashMap::<i32, f32>::new();
    map.insert(5, 7.0f32);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(5, 7.0f32)];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<i32, f32>::new();
    btree.insert(5, 7.0f32);
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }

}