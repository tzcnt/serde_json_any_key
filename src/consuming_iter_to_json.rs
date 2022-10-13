
use std::any::{Any, TypeId};
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};

/// Blanket impl [into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map) for all `IntoIterator<Item=(K,V)>` types.
pub trait ConsumingIterToJson<'a,K,V>: IntoIterator<Item=(K,V)> where
Self: Sized,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<(K,V)>` to a JSON map. This includes, but is not limited to, the following example types:  
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
    serde_json::to_string(&SerializeConsumingIterWrapper {
      iter: RefCell::new(self.into_iter())
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(K,V)>> ConsumingIterToJson<'a,K,V> for T where
T: IntoIterator<Item=(K,V)>,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }

struct SerializeConsumingIterWrapper<K,V,I> where
I: Iterator<Item=(K,V)>,
{
  pub iter: RefCell<I>
}

impl<K,V,I> Serialize for SerializeConsumingIterWrapper<K,V,I> where
  I: Iterator<Item=(K,V)>,
  K: Serialize + Any,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    // compiler seems to be able to optimize this branch away statically
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
