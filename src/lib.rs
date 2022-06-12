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
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};
use serde::de::{Deserialize, Deserializer, Visitor};

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

pub struct JsonToTupleIter<K,V> {
  iter: serde_json::map::IntoIter,
  k: std::marker::PhantomData<K>,
  v: std::marker::PhantomData<V>
}

pub fn json_to_iter<K,V>(str: &str) -> Result<JsonToTupleIter<K,V>, serde_json::Error> {
  let json_value = serde_json::from_str(&str)?;
  let json_map = match json_value {
    serde_json::Value::Object(map) => map,
          _ => { return Err(serde_json::Error::custom("Value is not a JSON map")); },
  };
  Ok(JsonToTupleIter {
    iter: json_map.into_iter(),
    k: std::marker::PhantomData,
    v: std::marker::PhantomData
  })
}

impl<K,V> Iterator for JsonToTupleIter<K,V> where
for<'de> K: Deserialize<'de> + Any,
for<'de> V: Deserialize<'de>
{
  type Item = Result<(K,V), serde_json::Error>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.iter.next() {
      Some(a) => {
        let key_obj: K = match TypeId::of::<K>() == TypeId::of::<String>() {
          true => match <K as Deserialize>::deserialize(serde_json::Value::from(a.0)) {
            Ok(k) => k,
            Err(e) => { return Some(Err(e)); }
          },
          false => match serde_json::from_str(&a.0) {
            Ok(k) => k,
            Err(e) => { return Some(Err(e)); }
          }
        };
        let val_obj: V = match <V as Deserialize>::deserialize(a.1) {
          Ok(v) => v,
          Err(e) => { return Some(Err(e)); }
        };
        Some(Ok((key_obj, val_obj)))
      },
      None => None
    }
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

pub mod any_key_map {
use super::*;
use serde::de::{MapAccess};
use std::fmt;

  pub fn serialize<'s, S, C, K, V>(coll: C, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer,
  C: IntoIterator<Item=(&'s K,&'s V)>,
  K: Serialize + Any + 's,
  V: Serialize + 's
  {
    let mut iter = coll.into_iter();
    let wrap = SerializeMapIterWrapper {
      iter: RefCell::new(&mut iter),
    };
    wrap.serialize(serializer)
  }

  pub(crate) struct MapIter<'de, A, K, V> {
    pub(crate) access: A,
    marker: PhantomData<(&'de (), K, V)>,
  }

  impl<'de, A, K, V> MapIter<'de, A, K, V> {
      pub(crate) fn new(access: A) -> Self
      where
          A: MapAccess<'de>,
      {
          Self {
              access,
              marker: PhantomData,
          }
      }
  }

  impl<'de, A, K, V> Iterator for MapIter<'de, A, K, V>
  where
      A: MapAccess<'de>,
      K: Deserialize<'de>,
      V: Deserialize<'de>,
  {
      type Item = Result<(K, V), A::Error>;

      fn next(&mut self) -> Option<Self::Item> {
          self.access.next_entry().transpose()
      }

      fn size_hint(&self) -> (usize, Option<usize>) {
          match self.access.size_hint() {
              Some(size) => (size, Some(size)),
              None => (0, None),
          }
      }
  }

  pub fn deserialize<'d, D, C, K, V>(deserializer: D) -> Result<C, D::Error> where
    D: Deserializer<'d>,
    C: FromIterator<(K,V)> + Sized,
    for<'de> K: Deserialize<'de> + Any + 'd,
    for<'de> V: Deserialize<'de> + 'd,
  {
    struct Helper<K,V>(PhantomData<(K,V)>);
    impl<'d,K,V> Visitor<'d> for Helper<K,V>
    where
    for<'de> K: Deserialize<'de> + Any + 'd,
    for<'de> V: Deserialize<'de> + 'd
    {
        type Value = Vec<(K,V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a sequence")
        }

        fn visit_map<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'d>,
        {
          let mut vec = Vec::<(K,V)>::new();
          for item in MapIter::<'d, A, String, V>::new(seq)
              .map(|res| {
                res.and_then(|value: (String,V)| {
                  let key_obj: K = match TypeId::of::<K>() == TypeId::of::<String>() {
                    true => match <K as Deserialize>::deserialize(serde_json::Value::from(value.0)) {
                      Ok(k) => k,
                      Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                    },
                    false => match serde_json::from_str(&value.0) {
                      Ok(k) => k,
                      Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                    }
                  };
                  Ok((key_obj, value.1))
                })
              }) {
                vec.push(item?);
              }
          Ok(vec)
        }
    }
    
    deserializer
        .deserialize_map(Helper(PhantomData))
        .map(C::from_iter)
  }
}

#[allow(dead_code)]
pub mod any_key_vec {
  use super::*;
  use serde::de::{MapAccess};
  use std::fmt;
  pub fn serialize<'s, S, C, K, V>(coll: C, serializer: S) -> Result<S::Ok, S::Error>
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

  pub(crate) struct MapIter<'de, A, K, V> {
    pub(crate) access: A,
    marker: PhantomData<(&'de (), K, V)>,
  }

  impl<'de, A, K, V> MapIter<'de, A, K, V> {
      pub(crate) fn new(access: A) -> Self
      where
          A: MapAccess<'de>,
      {
          Self {
              access,
              marker: PhantomData,
          }
      }
  }

  impl<'de, A, K, V> Iterator for MapIter<'de, A, K, V>
  where
      A: MapAccess<'de>,
      K: Deserialize<'de>,
      V: Deserialize<'de>,
  {
      type Item = Result<(K, V), A::Error>;

      fn next(&mut self) -> Option<Self::Item> {
          let ret = self.access.next_entry().transpose();
          let y = ret;
          return y;
      }

      fn size_hint(&self) -> (usize, Option<usize>) {
          match self.access.size_hint() {
              Some(size) => (size, Some(size)),
              None => (0, None),
          }
      }
  }

  pub fn deserialize<'d, D, C, K, V>(deserializer: D) -> Result<C, D::Error> where
    D: Deserializer<'d>,
    C: FromIterator<(K,V)> + Sized,
    for<'de> K: Deserialize<'de> + Any + 'd,
    for<'de> V: Deserialize<'de> + 'd,
  {
    struct Helper<K,V>(PhantomData<(K,V)>);
    impl<'d,K,V> Visitor<'d> for Helper<K,V>
    where
    for<'de> K: Deserialize<'de> + Any + 'd,
    for<'de> V: Deserialize<'de> + 'd
    {
        type Value = Vec<(K,V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a sequence")
        }

        fn visit_map<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'d>,
        {
          let mut vec = Vec::<(K,V)>::new();
          for item in MapIter::<'d, A, String, V>::new(seq)
              .map(|res| {
                res.and_then(|value: (String,V)| {
                  let key_obj: K = match TypeId::of::<K>() == TypeId::of::<String>() {
                    true => match <K as Deserialize>::deserialize(serde_json::Value::from(value.0)) {
                      Ok(k) => k,
                      Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                    },
                    false => match serde_json::from_str(&value.0) {
                      Ok(k) => k,
                      Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                    }
                  };
                  Ok((key_obj, value.1))
                    //value.parse::<S>().map_err(Error::custom)
                })
              }) {
                vec.push(item?);
              }
          Ok(vec)
        }
    }
    
    deserializer
        .deserialize_map(Helper(PhantomData))
        .map(C::from_iter)
  }
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

  #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
  pub struct TestWithString {
    pub a: i32,
    pub b: i32,
    pub c: String
  }
  
  #[test]
  fn test_nested_struct_serde_1_level() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map")]
      pub inner: HashMap<TestWithString,TestWithString>
    }
    
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(TestWithString,TestWithString)>
    }

    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct Outer {
      pub map: SerdeWithMap,
      pub vec: SerdeWithVec
    }

    let mut map = SerdeWithMap { inner: HashMap::new() };
    map.inner.insert(TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()});

    let mut vec = SerdeWithVec { inner: vec![] };
    vec.inner.push((TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()}));

    let outer = Outer {
      map: map,
      vec: vec
    };

    let serialized = serde_json::to_string(&outer).unwrap();
    let deser: Outer = serde_json::from_str(&serialized).unwrap();
    assert_eq!(outer, deser);
  }

  #[test]
  fn test_nested_struct_serde_2_level() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map")]
      pub inner: HashMap<TestWithString,TestWithString>
    }
    
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug, Hash)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(TestWithString,TestWithString)>
    }

    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct Outer {
      #[serde(with = "any_key_map")]
      pub map: HashMap<SerdeWithVec, SerdeWithMap>,
      #[serde(with = "any_key_vec")]
      pub vec: Vec<(SerdeWithMap, SerdeWithVec)>
    }

    let mut map = SerdeWithMap { inner: HashMap::new() };
    map.inner.insert(TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()});

    let mut vec = SerdeWithVec { inner: vec![] };
    vec.inner.push((TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()}));

    let mut outer_map = HashMap::<SerdeWithVec, SerdeWithMap>::new();
    outer_map.insert(vec.clone(), map.clone());
    let mut outer_vec = Vec::<(SerdeWithMap, SerdeWithVec)>::new();
    outer_vec.push((map, vec));
    let outer = Outer {
      map: outer_map,
      vec: outer_vec
    };

    let serialized = serde_json::to_string(&outer).unwrap();
    let deser: Outer = serde_json::from_str(&serialized).unwrap();
    assert_eq!(outer, deser);
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